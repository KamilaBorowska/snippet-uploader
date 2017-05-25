#![feature(custom_derive, plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate r2d2;
extern crate r2d2_diesel;
#[macro_use]
extern crate serde_derive;
extern crate bcrypt;
#[macro_use]
extern crate error_chain;
extern crate ipnetwork;

mod db;
mod pages;
mod static_file;

use rocket::Data;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket_contrib::Template;

use db::init_pool;
use db::user::UserId;
use pages::{login, register};

#[derive(Serialize)]
struct Context<'a, T> {
    title: &'a str,
    flash: Option<&'a str>,
    page: T,
}

#[post("/upload", format = "multipart/form-data", data = "<data>")]
fn upload(user_id: UserId, data: Data) -> &'static str {
    "Przyjęto"
}

#[get("/")]
fn main_page(user_id: UserId, flash: Option<FlashMessage>) -> Template {
    let message = flash.as_ref().map(|f| f.msg());
    Template::render("upload",
                     &Context {
                         title: "Strona główna",
                         flash: message,
                         page: (),
                     })
}

#[get("/", rank = 2)]
fn login_redirect() -> Flash<Redirect> {
    Flash::error(Redirect::to("/login"), "Ta strona wymaga zalogowania.")
}

fn main() {
    rocket::ignite()
        .manage(init_pool())
        .attach(Template::fairing())
        .mount("/",
               routes![main_page, login_redirect, upload, static_file::static_file])
        .mount("/login", login::routes())
        .mount("/register", register::routes())
        .launch();
}
