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

use diesel::prelude::*;
use db::{init_pool, Connection};
use db::schema::files;
use db::user::UserId;
use pages::{login, register};

use std::error::Error;
use std::io::{self, Write};
use std::fs::{self, File};
use std::path::Path;

#[derive(Serialize)]
struct Context<'a, T> {
    title: &'a str,
    flash: Option<&'a str>,
    page: T,
}

#[post("/upload", format = "multipart/form-data", data = "<data>")]
fn upload(connection: Connection, user_id: UserId, data: Data) -> Result<Flash<Redirect>, Box<Error>> {
    // temporary, no multipart/form-data support so far
    let name = Path::new("plik.txt");
    let contents = b"zawartosc\r\n";

    let cow_name;
    let file_name = if let Some(name) = name.file_name() {
        cow_name = name.to_string_lossy();
        &*cow_name
    } else {
        return Ok(Flash::error(Redirect::to("/"), "Plik nie ma właściwej nazwy."));
    };

    let upload_path = Path::new("uploads").join(user_id.0.to_string());

    fs::create_dir_all(&upload_path)?;
    File::create(upload_path.join(file_name))?.write(contents)?;

    #[derive(Insertable)]
    #[table_name="files"]
    struct NewFile<'a> {
        name: &'a str,
        user_id: i32,
    }

    diesel::insert(&NewFile {
                       name: file_name,
                       user_id: user_id.0,
                   })
            .into(files::table)
            .execute(&*connection)?;

    Ok(Flash::success(Redirect::to("/"), "Plik został wrzucony."))
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
