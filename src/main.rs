#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all)]

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

mod api;
mod db;
mod index;
mod models;
mod settings;
mod storage;
mod templates;
mod ui;

// fn rocket() -> Result<Rocket> {
//     let settings = settings::load()?;

//     let config = rocket::config::Config {
//         port: settings.port,
//         ..rocket::config::Config::default()
//     };

//     Ok(rocket::custom(config)
//         .attach(db::DbConn::fairing())
//         .attach(db::DbMigrations::fairing())
//         .attach(SpaceHelmet::default())
//         .manage(settings)
//         .mount("/", routes![ui::routes::index, ui::routes::me])
//         .mount(
//             "/api/v1/crates",
//             routes![
//                 api::routes::crates_new,
//                 api::routes::yank,
//                 api::routes::unyank,
//                 api::routes::list_owners,
//                 api::routes::add_owners,
//                 api::routes::remove_owners,
//                 api::routes::search,
//                 api::routes::download,
//             ],
//         ))
// }

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,asgard=trace,warp=debug")
        .with_span_events(FmtSpan::CLOSE)
        .init();

    launch_warp().await
}

// async fn launch_rocket() -> Result<()> {
//     rocket()?
//         .launch()
//         .await
//         .map_err(|e| anyhow::anyhow!(e.to_string()))
// }

async fn launch_warp() -> Result<()> {
    let settings = settings::load()?;

    let pool = db::create_pool()?;
    db::run_migrations(pool.get()?)?;

    let index = Arc::new(index::new(&settings.index)?);
    let storage = Arc::new(Mutex::new(storage::new(&settings.storage.location)));

    let routes = api::filters::api(index, storage).or(ui::filters::ui());

    warp::serve(routes)
        .run(([127, 0, 0, 1], settings.port))
        .await;

    Ok(())
}
