use rocket::fairing::{AdHoc, Fairing};

use super::DbConn;

embed_migrations!("migrations");

pub struct DbMigrations;

impl DbMigrations {
    pub fn fairing() -> impl Fairing {
        AdHoc::on_attach("Database Migrations", |mut rocket| async {
            if let Some(conn) = DbConn::get_one(rocket.inspect().await) {
                if let Err(e) = embedded_migrations::run(&*conn) {
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
