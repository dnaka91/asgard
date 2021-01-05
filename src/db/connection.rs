use std::ops::{Deref, DerefMut};

use r2d2::{ManageConnection, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rocket::fairing::{AdHoc, Fairing};
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest};
use rocket::{Request, Rocket, State};
use rusqlite::Connection;

fn init_connection(conn: &mut Connection) -> Result<(), rusqlite::Error> {
    conn.pragma_update(None, "busy_timeout", &1000)?;
    conn.pragma_update(None, "foreign_keys", &"ON")?;
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "synchronous", &"NORMAL")?;
    conn.pragma_update(None, "wal_autocheckpoint", &1000)?;
    conn.pragma_update(None, "wal_checkpoint", &"TRUNCATE")?;
    Ok(())
}

struct DbConnPool(Pool<SqliteConnectionManager>);

pub struct DbConn(PooledConnection<SqliteConnectionManager>);

impl DbConn {
    pub fn fairing() -> impl Fairing {
        AdHoc::on_attach("Database Pool", |rocket| async {
            let manager = if cfg!(test) {
                SqliteConnectionManager::memory()
            } else {
                SqliteConnectionManager::file("data.db")
            }
            .with_init(init_connection);

            // First create a single connection to make sure all eventually locking PRAGMAs are run,
            // so we don't get any errors when spinning up the pool.
            if let Err(e) = manager.connect() {
                rocket::logger::error(&format!("Failed to initialize database\n{:?}", e));
                return Err(rocket);
            }

            let pool = Pool::builder().build(manager);

            match pool {
                Ok(p) => Ok(rocket.manage(DbConnPool(p))),
                Err(e) => {
                    rocket::logger::error(&format!("Failed to initialize database pool\n{:?}", e));
                    Err(rocket)
                }
            }
        })
    }

    pub fn get_one(rocket: &Rocket) -> Option<Self> {
        rocket
            .state::<DbConnPool>()
            .and_then(|pool| pool.0.get().ok())
            .map(Self)
    }
}

impl Deref for DbConn {
    type Target = Connection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DbConn {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    async fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let guard = request.guard::<State<'_, DbConnPool>>();
        let pool = rocket::try_outcome!(guard.await).0.clone();

        tokio::task::spawn_blocking(move || match pool.get() {
            Ok(conn) => Outcome::Success(Self(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        })
        .await
        .expect("failed to spawn a blocking task to get a pooled connection")
    }
}
