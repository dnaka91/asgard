#![allow(unused_variables)]

use std::io::Cursor;
use std::path::Path;

use log::error;
use maplit::btreeset;
use rocket::data::{FromDataFuture, FromDataSimple};
use rocket::http::{ContentType, Status};
use rocket::request::State;
use rocket::response::{self, NamedFile, Responder, Response};
use rocket::{delete, get, put, Data, Outcome, Request};
use rocket_contrib::json::Json;
use tokio::prelude::*;

use super::models::{
    AddOwnersRequest, AddOwnersResponse, Crate, ErrorDetail, ErrorResponse, ListOwnersResponse,
    Meta, PublishRequest, PublishResponse, RemoveOwnersRequest, RemoveOwnersResponse,
    SearchResponse, UnyankResponse, User, Warnings, YankResponse,
};

use crate::index::{self, Service as IndexService};
use crate::settings::Settings;

type Result<T> = anyhow::Result<T, ServerError>;

#[derive(Debug)]
pub struct ServerError(anyhow::Error);

impl<T: Into<anyhow::Error>> From<T> for ServerError {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl<'r> Responder<'r, 'static> for ServerError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        error!("{:?}", self.0);

        let message = ErrorResponse {
            errors: vec![ErrorDetail {
                detail: self.0.to_string(),
            }],
        };

        let mut resp = Response::build();

        if let Ok(json) = serde_json::to_vec(&message) {
            resp.header(ContentType::JSON)
                .sized_body(json.len(), Cursor::new(json));
        }

        resp.status(Status::InternalServerError).ok()
    }
}

pub struct PublishRequestWithData(PublishRequest, Vec<u8>);

impl FromDataSimple for PublishRequestWithData {
    type Error = anyhow::Error;

    fn from_data(request: &Request<'_>, data: Data) -> FromDataFuture<'static, Self, Self::Error> {
        Box::pin(async move {
            let mut stream = data.open();
            let mut len_buf = [0; 4];

            if let Err(e) = stream.read_exact(&mut len_buf).await {
                return Outcome::Failure((Status::UnprocessableEntity, e.into()));
            }

            let len = u32::from_le_bytes(len_buf);
            let mut buf = vec![0; len as usize];

            if let Err(e) = stream.read_exact(&mut buf).await {
                return Outcome::Failure((Status::UnprocessableEntity, e.into()));
            }

            let request = match serde_json::from_slice(&buf) {
                Ok(r) => r,
                Err(e) => {
                    return Outcome::Failure((Status::UnprocessableEntity, e.into()));
                }
            };

            if let Err(e) = stream.read_exact(&mut len_buf).await {
                return Outcome::Failure((Status::UnprocessableEntity, e.into()));
            }

            let len = u32::from_le_bytes(len_buf);
            buf.resize_with(len as usize, Default::default);

            if let Err(e) = stream.read_exact(&mut buf).await {
                return Outcome::Failure((Status::UnprocessableEntity, e.into()));
            }

            Outcome::Success(Self(request, buf))
        })
    }
}

#[put("/new", data = "<data>")]
pub fn crates_new(
    data: PublishRequestWithData,
    settings: State<Settings>,
) -> Result<Json<PublishResponse>> {
    let index_service = index::new(&settings.index.location)?;

    index_service.add_crate(data.0)?;

    Ok(Json(PublishResponse {
        warnings: Warnings {
            invalid_categories: btreeset![],
            invalid_badges: btreeset![],
            other: vec![],
        },
    }))
}

#[delete("/<crate_name>/<version>/yank")]
pub fn yank(
    crate_name: String,
    version: String,
    settings: State<Settings>,
) -> Result<Json<YankResponse>> {
    let index_service = index::new(&settings.index.location)?;

    index_service.yank(crate_name.parse()?, version.parse()?, true)?;

    Ok(Json(YankResponse { ok: true }))
}

#[put("/<crate_name>/<version>/unyank")]
pub fn unyank(
    crate_name: String,
    version: String,
    settings: State<Settings>,
) -> Result<Json<UnyankResponse>> {
    let index_service = index::new(&settings.index.location)?;

    index_service.yank(crate_name.parse()?, version.parse()?, false)?;

    Ok(Json(UnyankResponse { ok: true }))
}

#[get("/<crate_name>/owners")]
pub fn list_owners(crate_name: String) -> Json<ListOwnersResponse> {
    Json(ListOwnersResponse {
        users: vec![User {
            id: 70,
            login: "github:rust-lang:core".to_owned(),
            name: Some("Core".to_owned()),
        }],
    })
}

#[put("/<crate_name>/owners", data = "<req>")]
pub fn add_owners(crate_name: String, req: Json<AddOwnersRequest>) -> Json<AddOwnersResponse> {
    Json(AddOwnersResponse {
        ok: true,
        msg: "user ehuss has been invited to be an owner of crate cargo".to_owned(),
    })
}

#[delete("/<crate_name>/owners", data = "<req>")]
pub fn remove_owners(
    crate_name: String,
    req: Json<RemoveOwnersRequest>,
) -> Json<RemoveOwnersResponse> {
    Json(RemoveOwnersResponse {
        ok: true,
        msg: "".to_owned(),
    })
}

#[get("/?<q>&<per_page>")]
pub fn search(q: String, per_page: u8) -> Json<SearchResponse> {
    Json(SearchResponse {
        crates: vec![Crate {
            name: "rand".parse().unwrap(),
            max_version: "0.6.1".parse().unwrap(),
            description: "Random number generators and other randomness functionality.\n"
                .to_owned(),
        }],
        meta: Meta { total: 119 },
    })
}

#[get("/<crate_name>/<version>/download")]
pub fn download(crate_name: String, version: String) -> NamedFile {
    todo!()
}
