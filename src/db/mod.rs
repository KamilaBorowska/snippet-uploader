pub mod schema;
pub mod user;

use std::ops::Deref;

use diesel::pg::PgConnection;
use diesel::r2d2::{Pool, PooledConnection, ConnectionManager};

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};

pub fn init_pool() -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::new("postgresql://");
    Pool::new(manager).expect("db pool")
}

pub struct Connection(PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for Connection {
    type Target = PgConnection;

    fn deref(&self) -> &PgConnection {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Connection {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Connection, ()> {
        let pool = match <&State<Pool<_>>>::from_request(request).await {
            Outcome::Success(pool) => pool,
            Outcome::Failure(e) => return Outcome::Failure(e),
            Outcome::Forward(_) => return Outcome::Forward(()),
        };

        match pool.get() {
            Ok(conn) => Outcome::Success(Connection(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}
