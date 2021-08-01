use warp::{http::StatusCode, reject::Reject, Rejection, Reply};

use super::models::{ErrorDetail, ErrorResponse};

pub type Result<T, E = Rejection> = std::result::Result<T, E>;

#[derive(Debug, derive_more::Display)]
pub struct ServerError(pub anyhow::Error);

impl<T: Into<anyhow::Error>> From<T> for ServerError {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Reject for ServerError {}

pub async fn recover(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(ServerError(err)) = err.find() {
        let mut errors = Vec::new();
        let mut current: Option<&dyn std::error::Error> = Some(err.as_ref());

        while let Some(err) = current {
            errors.push(ErrorDetail {
                detail: err.to_string(),
            });
            current = err.source();
        }

        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse { errors }),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    Err(err)
}
