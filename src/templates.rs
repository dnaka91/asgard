use std::io::Cursor;

use askama::Template;
use rocket::http::{ContentType, Status};
use rocket::response::{Responder, Result};
use rocket::{Request, Response};

macro_rules! responder {
    ($($name:ident),+) => {
        $(
            #[rocket::async_trait]
            impl<'r> Responder<'r> for $name {
                async fn respond_to(self, _: &'r Request<'_>) -> Result<'r> {
                    let ext = self.extension().unwrap_or_else(|| "html");
                    let resp = self.render().map_err(|_| Status::InternalServerError)?;
                    let ctype = ContentType::from_extension(ext).ok_or(Status::InternalServerError)?;

                    Response::build()
                        .header(ctype)
                        .sized_body(Cursor::new(resp))
                        .await
                        .ok()
                }
            }
        )+
    };
}

responder!(Index, Me);

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;

#[derive(Template)]
#[template(path = "me.html")]
pub struct Me;
