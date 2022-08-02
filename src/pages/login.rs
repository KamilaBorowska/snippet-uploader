use rocket::Route;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::request::{FlashMessage};
use rocket::response::{Debug, Flash, Redirect};
use rocket_dyn_templates::Template;

use diesel;
use diesel::prelude::*;
use crate::db::Connection;

use crate::db::user::{self, Error, LoginForm, UserId};

use sha2::{Sha256, Digest};

use crate::Context;

use std::net::SocketAddr;

fn csrf_token_for(address: &SocketAddr) -> String {
    let mut hasher = Sha256::default();
    hasher.input(b"GERNVEWffewjomfewomoewnvikrnv53858328r");
    hasher.input(address.ip().to_string().as_bytes());
    hasher.input(b"EWGGgjrwgvmewogn32ng3otno3gjo3whgo4hgo4hj90ge0wsm0f");
    hasher.result().iter().map(|v| format!("{:02x}", v)).collect()
}

#[post("/", data = "<form>")]
fn index(cookie_jar: &CookieJar<'_>,
         form: Form<LoginForm>,
         address: SocketAddr,
         db: Connection)
         -> Result<Flash<Redirect>, Debug<user::Error>> {
    let user = form.into_inner();
    if csrf_token_for(&address) != user.csrf {
        return Ok(Flash::error(Redirect::to("/login"), "Nie ma tokenu CSRF"));
    }
    match user.login(&db, address) {
        Ok(id) => {
            id.login(cookie_jar, &db)?;
            Ok(Flash::success(Redirect::to("/"), "Zalogowano."))
        }
        Err(e) => {
            Ok(Flash::error(Redirect::to("/login"),
                            match *e.kind() {
                                user::ErrorKind::Query(diesel::result::Error::NotFound) => "Niepoprawny login.",
                                user::ErrorKind::InvalidUserOrPassword => "Niepoprawny login lub hasło.",
                                _ => return Err(e.into()),
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
    let message = flash.as_ref().map(|f| f.message());
    let csrf_token = csrf_token_for(&address);
    Template::render("login",
                     &Context {
                          title: "Logowanie",
                          flash: message,
                          page: csrf_token,
                      })
}

#[get("/logout")]
fn logout(user: UserId, connection: Connection) -> Result<Redirect, Debug<Error>> {
    use crate::db::schema::sessions::dsl::*;
    diesel::delete(sessions.filter(user_id.eq(user.0))).execute(&*connection).map_err(|e| Debug(e.into()))?;
    Ok(Redirect::to("/"))
}

pub fn routes() -> Vec<Route> {
    routes![index, redirect, page, logout]
}
