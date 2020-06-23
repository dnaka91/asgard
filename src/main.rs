#![forbid(unsafe_code)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use rocket::{routes, Rocket};
use rocket_contrib::helmet::SpaceHelmet;

mod api;
mod db;
mod index;
mod templates;
mod ui;
mod models;

#[rocket::launch]
fn rocket() -> Rocket {
    rocket::ignite()
        .attach(db::DbConn::fairing())
        .attach(db::DbMigrations::fairing())
        .attach(SpaceHelmet::default())
        .mount("/", routes![ui::routes::index, ui::routes::me])
        .mount(
            "/api/v1/crates",
            routes![
                api::routes::crates_new,
                api::routes::yank,
                api::routes::unyank,
                api::routes::list_owners,
                api::routes::add_owners,
                api::routes::remove_owners,
                api::routes::search,
            ],
        )
}
