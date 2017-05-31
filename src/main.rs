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
extern crate multipart;

mod db;
mod pages;
mod static_file;
mod upload;

use rocket::request::FlashMessage;
use rocket::response::{Flash, NamedFile, Redirect};
use rocket_contrib::Template;

use diesel::prelude::*;
use db::{init_pool, Connection};
use db::schema::files;
use db::user::UserId;
use pages::{login, register, upload as upload_page};

use std::error::Error;
use std::io;
use std::path::Path;

#[derive(Serialize)]
struct Context<'a, T> {
    title: &'a str,
    flash: Option<&'a str>,
    page: T,
}

#[get("/")]
fn main_page(connection: Connection,
             user_id: UserId,
             flash: Option<FlashMessage>)
             -> Result<Template, Box<Error>> {
    use db::schema::files::dsl;
    let message = flash.as_ref().map(|f| f.msg());
    let files = files::table.filter(dsl::user_id.eq(user_id.0))
        .select(dsl::name)
        .load(&*connection)?;

    #[derive(Serialize)]
    struct UploadPage {
        files: Vec<String>,
        user_id: i32,
    }

    Ok(Template::render("upload",
                        &Context {
                             title: "Strona główna",
                             flash: message,
                             page: UploadPage {
                                 files: files,
                                 user_id: user_id.0,
                             },
                         }))
}

#[get("/", rank = 2)]
fn login_redirect() -> Flash<Redirect> {
    Flash::error(Redirect::to("/login"), "Ta strona wymaga zalogowania.")
}

#[get("/files/<user_id>/<path>")]
fn download(user_id: i32, path: String) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("uploads/")
                        .join(user_id.to_string())
                        .join(path))
}

fn main() {
    rocket::ignite()
        .manage(init_pool())
        .attach(Template::fairing())
        .mount("/",
               routes![main_page, login_redirect, download, static_file::static_file])
        .mount("/login", login::routes())
        .mount("/register", register::routes())
        .mount("/upload", upload_page::routes())
        .launch();
}
