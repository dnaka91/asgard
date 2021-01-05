use rocket::fairing::{AdHoc, Fairing};

use super::DbConn;

mod embedded {
    refinery::embed_migrations!("migrations");
}

pub struct DbMigrations;

impl DbMigrations {
    pub fn fairing() -> impl Fairing {
        AdHoc::on_attach("Database Migrations", |rocket| async {
            if let Some(mut conn) = DbConn::get_one(&rocket) {
                if let Err(e) = embedded::migrations::runner().run(&mut *conn) {
                    rocket::logger::error(&format!("Database initialization failed: {:?}", e));
                    Err(rocket)
                } else {
                    Ok(rocket)
                }
            } else {
                rocket::logger::error("No database connection");
                Err(rocket)
            }
        })
    }
}
