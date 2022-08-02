use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::response::{Debug, Flash, Redirect};
use rocket::Route;

use crate::db::schema::files;
use crate::db::user::UserId;
use crate::db::Connection;
use diesel;
use diesel::prelude::*;

use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Insertable)]
#[table_name = "files"]
struct NewFile<'a> {
    name: &'a str,
    user_id: i32,
}

#[post("/", data = "<upload>")]
async fn upload(
    connection: Connection,
    user_id: UserId,
    mut upload: Form<TempFile<'_>>,
) -> Result<Flash<Redirect>, Debug<Box<dyn Error>>> {
    let name: String = if let Some(name) = upload.name() {
        name.into()
    } else {
        return Ok(Flash::error(
            Redirect::to("/"),
            "Plik nie ma właściwej nazwy.",
        ));
    };

    let upload_path = Path::new("uploads").join(user_id.0.to_string());

    fs::create_dir_all(&upload_path).map_err(|e| Debug(e.into()))?;
    upload
        .persist_to(upload_path.join(&name))
        .await
        .map_err(|e| Debug(e.into()))?;

    diesel::insert_into(files::table)
        .values(&NewFile {
            name: &name,
            user_id: user_id.0,
        })
        .execute(&*connection)
        .map_err(|e| Debug(e.into()))?;

    Ok(Flash::success(Redirect::to("/"), "Plik został wrzucony."))
}

pub fn routes() -> Vec<Route> {
    routes![upload]
}
