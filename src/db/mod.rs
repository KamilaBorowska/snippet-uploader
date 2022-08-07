pub mod schema;
pub mod user;

use rocket_sync_db_pools::database;
use rocket_sync_db_pools::diesel::PgConnection;

#[database("main")]
pub struct Connection(PgConnection);
