use rocket::Route;
use rocket::http::Session;
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket_contrib::Template;

use diesel;
use db::Connection;

use db::user::{self, LoginForm, UserId};

use Context;

use std::net::SocketAddr;

#[post("/", data = "<form>")]
fn index(mut session: Session,
         form: Form<LoginForm>,
         address: SocketAddr,
         db: Connection)
         -> user::Result<Flash<Redirect>> {
    let user = form.get();
    match user.login(&db, address) {
        Ok(id) => {
            id.login(&mut session)?;
            Ok(Flash::success(Redirect::to("/"), "Zalogowano."))
        }
        Err(e) => {
            Ok(Flash::error(Redirect::to("/login"), match *e.kind() {
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
fn page(flash: Option<FlashMessage>) -> Template {
    let message = flash.as_ref().map(|f| f.msg());
    Template::render("login",
                     &Context {
                         title: "Logowanie",
                         flash: message,
                         page: "",
                     })
}

pub fn routes() -> Vec<Route> {
    routes![index, redirect, page]
}
