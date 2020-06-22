use std::ops::Deref;

use diesel::r2d2::{
    ConnectionManager, CustomizeConnection, ManageConnection, Pool, PooledConnection,
};
use diesel::{Connection, SqliteConnection};
use rocket::fairing::{AdHoc, Fairing};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Manifest, Outcome, Request, State};

#[derive(Copy, Clone, Debug)]
struct ConnectionCustomizer;

impl<C> CustomizeConnection<C, diesel::r2d2::Error> for ConnectionCustomizer
where
    C: Connection,
{
    fn on_acquire(&self, conn: &mut C) -> Result<(), diesel::r2d2::Error> {
        conn.batch_execute(
            "\
            PRAGMA busy_timeout = 1000;\
            PRAGMA foreign_keys = ON;\
            PRAGMA journal_mode = WAL;\
            PRAGMA synchronous = NORMAL;\
            PRAGMA wal_autocheckpoint = 1000;\
            PRAGMA wal_checkpoint(TRUNCATE);\
            ",
        )
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

struct DbConnPool(Pool<ConnectionManager<SqliteConnection>>);

pub struct DbConn(PooledConnection<ConnectionManager<SqliteConnection>>);

impl DbConn {
    pub fn fairing() -> impl Fairing {
        AdHoc::on_attach("Database Pool", |rocket| async {
            let url = if cfg!(test) { ":memory:" } else { "data.db" };
            let manager = ConnectionManager::<SqliteConnection>::new(url);

            // First create a single connection to make sure all eventually locking PRAGMAs are run,
            // so we don't get any errors when spinning up the pool.
            if let Err(e) = manager
                .connect()
                .and_then(|mut conn| ConnectionCustomizer.on_acquire(&mut conn))
            {
                rocket::logger::error(&format!("Failed to initialize database\n{:?}", e));
                return Err(rocket);
            }

            let pool = Pool::builder()
                .connection_customizer(Box::new(ConnectionCustomizer))
                .build(manager);

            match pool {
                Ok(p) => Ok(rocket.manage(DbConnPool(p))),
                Err(e) => {
                    rocket::logger::error(&format!("Failed to initialize database pool\n{:?}", e));
                    Err(rocket)
                }
            }
        })
    }

    pub fn get_one(manifest: &Manifest) -> Option<Self> {
        manifest
            .state::<DbConnPool>()
            .and_then(|pool| pool.0.get().ok())
            .map(Self)
    }
}

impl Deref for DbConn {
    type Target = SqliteConnection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    async fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let guard = request.guard::<State<DbConnPool>>();
        let pool = rocket::try_outcome!(guard.await).0.clone();

        tokio::task::spawn_blocking(move || match pool.get() {
            Ok(conn) => Outcome::Success(Self(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        })
        .await
        .expect("failed to spawn a blocking task to get a pooled connection")
    }
}
