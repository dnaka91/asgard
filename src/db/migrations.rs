use anyhow::Result;

use super::DbConn;

mod embedded {
    refinery::embed_migrations!("migrations");
}

pub fn run(mut conn: DbConn) -> Result<()> {
    embedded::migrations::runner().run(&mut *conn)?;
    Ok(())
}
