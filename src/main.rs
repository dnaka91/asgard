#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all)]
#![allow(unused_imports)]

use anyhow::Result;
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

fn rocket() -> Result<Rocket> {
    let settings = settings::load().unwrap();

    let config = rocket::config::Config {
        port: settings.port,
        ..rocket::config::Config::default()
    };

    Ok(rocket::custom(config)
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
        ))
}

#[rocket::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,_=debug,crator=trace")
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    rocket()?
        .launch()
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}
