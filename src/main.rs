#![forbid(unsafe_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use rocket::{routes, Rocket};
use rocket_contrib::helmet::SpaceHelmet;

mod api;
mod db;
mod index;
mod models;
mod settings;
mod storage;
mod templates;
mod ui;

#[rocket::launch]
fn rocket() -> Rocket {
    let settings = settings::load().unwrap();

    let mut config = rocket::config::Config::active().unwrap();

    config.set_port(settings.port);

    rocket::custom(config)
        .attach(db::DbConn::fairing())
        .attach(db::DbMigrations::fairing())
        .attach(SpaceHelmet::default())
        .manage(settings)
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
                api::routes::download,
            ],
        )
}
