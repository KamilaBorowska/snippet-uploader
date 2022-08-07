#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate bcrypt;
#[macro_use]
extern crate error_chain;
extern crate chrono;
extern crate ipnetwork;
extern crate sha2;

mod db;
mod pages;
mod static_file;

use crate::db::schema::files;
use crate::db::user::UserId;
use crate::db::Connection;
use chrono::naive::NaiveDateTime;
use diesel::prelude::*;
use ipnetwork::IpNetwork;
use pages::{login, register, upload as upload_page};
use rocket::request::FlashMessage;
use rocket::response::{Debug, Flash, Redirect};
use rocket_dyn_templates::Template;
use serde::Serialize;
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
async fn main_page(
    connection: Connection,
    user_id: UserId,
    flash: Option<FlashMessage<'_>>,
) -> Result<Template, Debug<Box<dyn Error>>> {
    use crate::db::schema::files::dsl;
    let message = flash.as_ref().map(|f| f.message());
    let files = connection
        .run(move |c| {
            files::table
                .filter(dsl::user_id.eq(user_id.0))
                .select(dsl::name)
                .load(c)
        })
        .await
        .map_err(|e| Debug(e.into()))?;

    #[derive(Serialize)]
    struct UploadPage {
        files: Vec<String>,
        user_id: i32,
    }

    Ok(Template::render(
        "upload",
        &Context {
            title: "Strona główna",
            flash: message,
            page: UploadPage {
                files: files,
                user_id: user_id.0,
            },
        },
    ))
}

#[get("/logins")]
async fn display_logins(
    connection: Connection,
    user_id: UserId,
) -> Result<Template, Debug<Box<dyn Error + Sync + Send>>> {
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

    use crate::db::schema::logins::{dsl, table};
    let (successful_logins, unsuccessful_logins) = connection
        .run(move |c| {
            let successful_logins = table
                .filter(dsl::user_id.eq(user_id.0))
                .filter(dsl::successful.eq(true))
                .select((dsl::ip, dsl::time, dsl::successful))
                .order(dsl::time.desc())
                .limit(5)
                .load(c)?;

            let unsuccessful_logins = table
                .filter(dsl::user_id.eq(user_id.0))
                .filter(dsl::successful.eq(false))
                .select((dsl::ip, dsl::time, dsl::successful))
                .order(dsl::time.desc())
                .limit(10)
                .load(c)?;

            Ok((successful_logins, unsuccessful_logins))
        })
        .await
        .map_err(|e| Debug(e))?;

    Ok(Template::render(
        "logins",
        &Context {
            title: "Próby zalogowania się",
            flash: None,
            page: successful_logins
                .into_iter()
                .chain(unsuccessful_logins)
                .map(
                    |Row {
                         ip,
                         time,
                         successful,
                     }| {
                        TemplateRow {
                            ip: ip.ip().to_string(),
                            time: time.to_string(),
                            successful: successful,
                        }
                    },
                )
                .collect::<Vec<_>>(),
        },
    ))
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
    File::open(Path::new("uploads/").join(user_id.to_string()).join(path))
}

#[get("/files/<user_id>/<path>/display")]
fn file_display(user_id: i32, path: String) -> Result<Template, io::Error> {
    #[derive(Serialize)]
    struct FileMetadata {
        contents: String,
        file_name: String,
    }

    let mut contents = String::new();
    let mut file = File::open(Path::new("uploads/").join(user_id.to_string()).join(&path))?;
    file.read_to_string(&mut contents)?;
    Ok(Template::render(
        "file",
        &Context {
            title: "Wyświetlanie plików",
            flash: None,
            page: FileMetadata {
                contents: contents,
                file_name: path,
            },
        },
    ))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Connection::fairing())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                main_page,
                login_redirect,
                logins_login_redirect,
                display_logins,
                download,
                file_display,
                static_file::static_file
            ],
        )
        .mount("/login", login::routes())
        .mount("/register", register::routes())
        .mount("/upload", upload_page::routes())
}
