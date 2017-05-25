pub mod schema;
pub mod user;

use std::ops::Deref;

use diesel::pg::PgConnection;
use r2d2::{Config, Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};

pub fn init_pool() -> Pool<ConnectionManager<PgConnection>> {
    let config = Config::default();
    let manager = ConnectionManager::new("postgresql://");
    Pool::new(config, manager).expect("db pool")
}

pub struct Connection(PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for Connection {
    type Target = PgConnection;

    fn deref(&self) -> &PgConnection {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Connection {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Connection, ()> {
        let pool = match State::<Pool<_>>::from_request(request) {
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
