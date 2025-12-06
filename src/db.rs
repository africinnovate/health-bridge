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


// use diesel::{
//     r2d2::{self, ConnectionManager},
//     PgConnection,
// };
// use anyhow::Result;

// pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// pub fn init_pool(database_url: &str) -> Result<DbPool> {
//     let manager = ConnectionManager::<PgConnection>::new(database_url);
//     let pool = r2d2::Pool::builder()
//         .max_size(15)
//         .build(manager)?;
//     Ok(pool)
// }
