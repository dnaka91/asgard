use std::{collections::BTreeSet, convert::Infallible, sync::Arc};

use anyhow::ensure;
use hyper::{
    body::{Buf, Bytes},
    Body,
};
use semver::Version;
use tokio::{sync::Mutex, task};
use tokio_util::codec::{BytesCodec, FramedRead};
use tracing::instrument;
use warp::{reply::Response, Rejection, Reply};

use super::{
    error::{Result, ServerError},
    models::{
        AddOwnersRequest, AddOwnersResponse, Crate, ListOwnersResponse, Meta, PublishRequest,
        PublishResponse, RemoveOwnersRequest, RemoveOwnersResponse, SearchQuery, SearchResponse,
        UnyankResponse, User, Warnings, YankResponse,
    },
};
use crate::{index, models::CrateName, storage};

pub struct PublishRequestWithData(PublishRequest, Vec<u8>);

impl PublishRequestWithData {
    fn from_bytes(mut data: Bytes) -> anyhow::Result<Self> {
        let len = data.get_u32_le() as usize;
        ensure!(
            data.remaining() >= len,
            "expected at least {} bytes but only {} remaining",
            len,
            data.remaining()
        );

        let request = serde_json::from_slice(&data.slice(0..len))?;
        data.advance(len);

        let len = data.get_u32_le() as usize;
        ensure!(
            data.remaining() == len,
            "expected {} bytes but only {} remaining",
            len,
            data.remaining()
        );

        let buf = data.to_vec();

        Ok(Self(request, buf))
    }
}

#[instrument(skip(data, storage, index))]
pub async fn crates_new(
    data: Bytes,
    storage: Arc<Mutex<impl storage::Service>>,
    index: Arc<impl index::Service>,
) -> Result<impl Reply> {
    let data = PublishRequestWithData::from_bytes(data).map_err(ServerError)?;

    storage
        .lock()
        .await
        .store(&data.0.name, &data.0.vers, &data.1)
        .await
        .map_err(ServerError)?;

    task::spawn_blocking(move || index.add_crate(data.0, &data.1).map_err(ServerError))
        .await
        .map_err(|e| ServerError(e.into()))??;

    Ok(warp::reply::json(&PublishResponse {
        warnings: Warnings {
            invalid_categories: BTreeSet::new(),
            invalid_badges: BTreeSet::new(),
            other: Vec::new(),
        },
    }))
}

#[instrument(skip(index))]
pub async fn yank(
    name: CrateName,
    version: Version,
    index: Arc<impl index::Service>,
) -> Result<impl Reply, Infallible> {
    task::spawn_blocking(move || {
        index.yank(name, version, true).unwrap();
    });

    Ok(warp::reply::json(&YankResponse { ok: true }))
}

#[instrument(skip(index))]
pub async fn unyank(
    name: CrateName,
    version: Version,
    index: Arc<impl index::Service>,
) -> Result<impl Reply, Infallible> {
    task::spawn_blocking(move || {
        index.yank(name, version, false).unwrap();
    });

    Ok(warp::reply::json(&UnyankResponse { ok: true }))
}

#[instrument]
pub async fn list_owners(_name: CrateName) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&ListOwnersResponse {
        users: vec![User {
            id: 70,
            login: "github:rust-lang:core".to_owned(),
            name: Some("Core".to_owned()),
        }],
    }))
}

#[instrument(skip(_req))]
pub async fn add_owners(
    _name: CrateName,
    _req: AddOwnersRequest,
) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&AddOwnersResponse {
        ok: true,
        msg: "user ehuss has been invited to be an owner of crate cargo".to_owned(),
    }))
}

#[instrument(skip(_req))]
pub async fn remove_owners(
    _name: CrateName,
    _req: RemoveOwnersRequest,
) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&RemoveOwnersResponse {
        ok: true,
        msg: "".to_owned(),
    }))
}

#[instrument]
pub async fn search(_query: SearchQuery) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&SearchResponse {
        crates: vec![Crate {
            name: "rand".parse().unwrap(),
            max_version: "0.6.1".parse().unwrap(),
            description: "Random number generators and other randomness functionality.\n"
                .to_owned(),
        }],
        meta: Meta { total: 119 },
    }))
}

#[instrument(skip(storage))]
pub async fn download(
    name: CrateName,
    version: Version,
    storage: Arc<Mutex<impl storage::Service>>,
) -> Result<impl Reply, Rejection> {
    let file = storage.lock().await.get(&name, &version).await.unwrap();

    match file {
        Some(file) => {
            let stream = FramedRead::new(file, BytesCodec::new());
            let body = Body::wrap_stream(stream);

            Ok(Response::new(body))
        }
        None => Err(ServerError(
            anyhow::anyhow!("crate {} with version {} not found", name, version)
                .context("what")
                .context("happened?"),
        )
        .into()),
    }
}
