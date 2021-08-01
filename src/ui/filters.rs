use warp::{Filter, Rejection, Reply};

use super::handlers;

pub fn ui() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    index().or(me())
}

/// `GET /`
fn index() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::end().and(warp::get()).map(handlers::index)
}

/// `GET /me`
fn me() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("me").and(warp::get()).map(handlers::me)
}
