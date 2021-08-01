use crate::templates;

#[tracing::instrument]
pub fn index() -> templates::Index {
    templates::Index
}

#[tracing::instrument]
pub fn me() -> templates::Me {
    templates::Me
}
