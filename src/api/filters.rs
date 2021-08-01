use std::{convert::Infallible, sync::Arc};

use semver::Version;
use tokio::sync::Mutex;
use warp::{Filter, Rejection, Reply};

use super::{
    error, handlers,
    models::{AddOwnersRequest, RemoveOwnersRequest, SearchQuery},
};
use crate::{
    index::Service as IndexService, models::CrateName, storage::Service as StorageService,
};

/// All API related routes prefixed with `/api/v1/crates/...`.
pub fn api(
    index: Arc<impl IndexService>,
    storage: Arc<Mutex<impl StorageService>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("api" / "v1" / "crates" / ..).and(
        crates_new(Arc::clone(&index), Arc::clone(&storage))
            .or(yank(Arc::clone(&index)))
            .or(unyank(index))
            .or(list_owners())
            .or(add_owners())
            .or(remove_owners())
            .or(search())
            .or(download(storage)),
    )
}

/// `PUT /api/v1/crates/<crate_name>/new`
fn crates_new(
    index: Arc<impl IndexService>,
    storage: Arc<Mutex<impl StorageService>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("new")
        .and(warp::put())
        .and(warp::body::content_length_limit(10_000_000))
        .and(warp::body::bytes())
        .and(with_storage(storage))
        .and(with_index(index))
        .and_then(handlers::crates_new)
        .recover(error::recover)
}

/// `DELETE /api/v1/crates/<crate_name>/<version>/yank`
fn yank(
    index: Arc<impl IndexService>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!(CrateName / Version / "yank")
        .and(warp::delete())
        .and(with_index(index))
        .and_then(handlers::yank)
        .recover(error::recover)
}

/// `PUT /api/v1/crates/<crate_name>/<version>/unyank`
fn unyank(
    index: Arc<impl IndexService>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!(CrateName / Version / "unyank")
        .and(warp::put())
        .and(with_index(index))
        .and_then(handlers::unyank)
        .recover(error::recover)
}

/// `GET /api/v1/crates/<crate_name>/owners`
fn list_owners() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!(CrateName / "owners")
        .and(warp::get())
        .and_then(handlers::list_owners)
        .recover(error::recover)
}

/// `PUT /api/v1/crates/<crate_name>/owners`
fn add_owners() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!(CrateName / "owners")
        .and(warp::put())
        .and(warp::body::json::<AddOwnersRequest>())
        .and_then(handlers::add_owners)
        .recover(error::recover)
}

/// `DELETE /api/v1/crates/<crate_name>/owners`
fn remove_owners() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!(CrateName / "owners")
        .and(warp::delete())
        .and(warp::body::json::<RemoveOwnersRequest>())
        .and_then(handlers::remove_owners)
        .recover(error::recover)
}

/// `GET /api/v1/crates/?q=<query>&per_page=<per_page>`
fn search() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::end()
        .and(warp::get())
        .and(warp::query::<SearchQuery>())
        .and_then(handlers::search)
        .recover(error::recover)
}

/// `GET /api/v1/crates/<crate_name>/<version>/download`
fn download(
    storage: Arc<Mutex<impl StorageService>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!(CrateName / Version / "download")
        .and(warp::get())
        .and(with_storage(storage))
        .and_then(handlers::download)
        .recover(error::recover)
}

fn with_index(
    service: Arc<impl IndexService>,
) -> impl Filter<Extract = (Arc<impl IndexService>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&service))
}

fn with_storage(
    service: Arc<Mutex<impl StorageService>>,
) -> impl Filter<Extract = (Arc<Mutex<impl StorageService>>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&service))
}
