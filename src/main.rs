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
extern crate chrono;
extern crate sha2;

mod db;
mod pages;
mod static_file;
mod upload;

use rocket::request::FlashMessage;
use rocket::response::{Flash, NamedFile, Redirect};
use rocket_contrib::Template;
use chrono::naive::datetime::NaiveDateTime;
use ipnetwork::IpNetwork;

use diesel::prelude::*;
use db::{init_pool, Connection};
use db::schema::files;
use db::user::UserId;
use pages::{login, register, upload as upload_page};

use std::error::Error;
use std::fs::File;
use std::io::{self, Read};
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

#[get("/logins")]
fn display_logins(connection: Connection, user_id: UserId) -> Result<Template, Box<Error>> {
    #[derive(Queryable)]
    struct Row {
        ip: IpNetwork,
        time: NaiveDateTime,
        successful: bool,
    }

    #[derive(Serialize)]
    struct TemplateRow {
        ip: String,
        time: String,
        successful: bool,
    }

    use db::schema::logins::{dsl, table};
    let successful_logins = table.filter(dsl::user_id.eq(user_id.0))
        .filter(dsl::successful.eq(true))
        .select((dsl::ip, dsl::time, dsl::successful))
        .order((dsl::time.desc()))
        .limit(5)
        .load(&*connection)?;

    let unsuccessful_logins = table.filter(dsl::user_id.eq(user_id.0))
        .filter(dsl::successful.eq(false))
        .select((dsl::ip, dsl::time, dsl::successful))
        .order((dsl::time.desc()))
        .limit(10)
        .load(&*connection)?;

    Ok(Template::render("logins",
                        &Context {
                             title: "Próby zalogowania się",
                             flash: None,
                             page: successful_logins
                                 .into_iter()
                                 .chain(unsuccessful_logins)
                                 .map(|Row {
                                           ip,
                                           time,
                                           successful,
                                       }| {
                                          TemplateRow {
                                              ip: ip.ip().to_string(),
                                              time: time.to_string(),
                                              successful: successful,
                                          }
                                      })
                                 .collect::<Vec<_>>(),
                         }))
}

#[get("/", rank = 2)]
fn login_redirect() -> Flash<Redirect> {
    Flash::error(Redirect::to("/login"), "Ta strona wymaga zalogowania.")
}

#[get("/logins", rank = 2)]
fn logins_login_redirect() -> Flash<Redirect> {
    login_redirect()
}

#[get("/files/<user_id>/<path>")]
fn download(user_id: i32, path: String) -> io::Result<File> {
    File::open(Path::new("uploads/")
                        .join(user_id.to_string())
                        .join(path))
}

#[get("/files/<user_id>/<path>/display")]
fn file_display(user_id: i32, path: String) -> Result<Template, io::Error> {
    #[derive(Serialize)]
    struct FileMetadata {
        contents: String,
        file_name: String,
    }

    let mut contents = String::new();
    let mut file = File::open(Path::new("uploads/")
                                  .join(user_id.to_string())
                                  .join(&path))?;
    file.read_to_string(&mut contents)?;
    Ok(Template::render("file",
                        &Context {
                             title: "Wyświetlanie plików",
                             flash: None,
                             page: FileMetadata {
                                 contents: contents,
                                 file_name: path,
                             },
                         }))
}

fn main() {
    rocket::ignite()
        .manage(init_pool())
        .attach(Template::fairing())
        .mount("/",
               routes![main_page,
                       login_redirect,
                       logins_login_redirect,
                       display_logins,
                       download,
                       file_display,
                       static_file::static_file])
        .mount("/login", login::routes())
        .mount("/register", register::routes())
        .mount("/upload", upload_page::routes())
        .launch();
}
