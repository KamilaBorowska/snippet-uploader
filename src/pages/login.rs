use rocket::Route;
use rocket::http::Session;
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket_contrib::Template;

use diesel;
use diesel::prelude::*;
use db::Connection;

use db::user::{self, Error, LoginForm, UserId};

use sha2::{Sha256, Digest};

use Context;

use std::net::SocketAddr;
use std::hash::{Hash, Hasher, SipHasher};

fn csrf_token_for(address: &SocketAddr) -> String {
    let mut hasher = Sha256::default();
    hasher.input(b"GERNVEWffewjomfewomoewnvikrnv53858328r");
    hasher.input(address.ip().to_string().as_bytes());
    hasher.input(b"EWGGgjrwgvmewogn32ng3otno3gjo3whgo4hgo4hj90ge0wsm0f");
    hasher.result().iter().map(|v| format!("{:02x}", v)).collect()
}

#[post("/", data = "<form>")]
fn index(mut session: Session,
         form: Form<LoginForm>,
         address: SocketAddr,
         db: Connection)
         -> user::Result<Flash<Redirect>> {
    let user = form.get();
    if csrf_token_for(&address) != user.csrf {
        return Ok(Flash::error(Redirect::to("/login"), "Nie ma tokenu CSRF"));
    }
    match user.login(&db, address) {
        Ok(id) => {
            id.login(&mut session, &db)?;
            Ok(Flash::success(Redirect::to("/"), "Zalogowano."))
        }
        Err(e) => {
            Ok(Flash::error(Redirect::to("/login"),
                            match *e.kind() {
                                user::ErrorKind::Query(diesel::result::Error::NotFound) => "Niepoprawny login.",
                                user::ErrorKind::InvalidUserOrPassword => "Niepoprawny login lub hasło.",
                                _ => return Err(e),
                            }))
        }
    }
}

#[get("/")]
fn redirect(_user: UserId) -> Flash<Redirect> {
    Flash::error(Redirect::to("/"), "Jesteś już zalogowany.")
}

#[get("/", rank = 2)]
fn page(flash: Option<FlashMessage>, address: SocketAddr) -> Template {
    let message = flash.as_ref().map(|f| f.msg());
    let csrf_token = csrf_token_for(&address);
    Template::render("login",
                     &Context {
                          title: "Logowanie",
                          flash: message,
                          page: csrf_token,
                      })
}

#[get("/logout")]
fn logout(user: UserId, connection: Connection) -> Result<Redirect, Error> {
    use db::schema::sessions::dsl::*;
    diesel::delete(sessions.filter(user_id.eq(user.0))).execute(&*connection)?;
    Ok(Redirect::to("/"))
}

pub fn routes() -> Vec<Route> {
    routes![index, redirect, page, logout]
}
