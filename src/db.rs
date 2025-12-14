use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use anyhow::Result;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_db(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(10)
        .build(manager)?;

    Ok(pool)
}
