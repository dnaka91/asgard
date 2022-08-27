#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, clippy::all)]

use std::sync::Arc;

use anyhow::Result;
use opentelemetry::{
    global, runtime,
    sdk::{trace, Resource},
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_semantic_conventions::resource;
use settings::{Settings, Tracing};
use tokio::sync::Mutex;
use tracing::error;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};
use warp::Filter;

mod api;
mod db;
mod index;
mod models;
mod settings;
mod storage;
mod templates;
mod ui;

#[cfg(debug_assertions)]
const ADDRESS: [u8; 4] = [127, 0, 0, 1];
#[cfg(not(debug_assertions))]
const ADDRESS: [u8; 4] = [0, 0, 0, 0];

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
    let mut settings = settings::load()?;

    let opentelemetry = settings
        .tracing
        .take()
        .map(|settings: Tracing| {
            global::set_error_handler(|error| {
                error!(target: "opentelemetry", %error);
            })?;

            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(settings.otlp.endpoint),
                )
                .with_trace_config(trace::config().with_resource(Resource::new([
                    resource::SERVICE_NAME.string(env!("CARGO_CRATE_NAME")),
                    resource::SERVICE_VERSION.string(env!("CARGO_PKG_VERSION")),
                ])))
                .install_batch(runtime::Tokio)?;

            anyhow::Ok(tracing_opentelemetry::layer().with_tracer(tracer))
        })
        .transpose()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
        .with(opentelemetry)
        .with(EnvFilter::builder().parse("info,asgard=trace,warp=debug")?)
        .init();

    launch_warp(settings).await
}

// async fn launch_rocket() -> Result<()> {
//     rocket()?
//         .launch()
//         .await
//         .map_err(|e| anyhow::anyhow!(e.to_string()))
// }

async fn launch_warp(settings: Settings) -> Result<()> {
    let pool = db::create_pool()?;
    db::run_migrations(pool.get()?)?;

    let index = Arc::new(index::new(&settings.index)?);
    let storage = Arc::new(Mutex::new(storage::new(&settings.storage.location)));

    let routes = api::filters::api(index, storage).or(ui::filters::ui());

    warp::serve(routes).run((ADDRESS, settings.port)).await;

    Ok(())
}
