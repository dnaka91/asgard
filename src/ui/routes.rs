use rocket::get;

use crate::templates;

#[get("/")]
pub fn index() -> templates::Index {
    templates::Index
}

#[get("/me")]
pub fn me() -> templates::Me {
    templates::Me
}
