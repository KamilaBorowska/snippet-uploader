use rocket::Route;
use rocket::response::{Flash, Redirect};

use diesel;
use diesel::prelude::*;
use db::Connection;
use db::schema::files;
use db::user::UserId;

use upload::FileUpload;

use std::error::Error;
use std::io::Write;
use std::fs::{self, File};
use std::path::Path;

#[derive(Insertable)]
#[table_name="files"]
struct NewFile<'a> {
    name: &'a str,
    user_id: i32,
}

#[post("/", data = "<upload>")]
fn upload(connection: Connection,
          user_id: UserId,
          upload: FileUpload)
          -> Result<Flash<Redirect>, Box<Error>> {

    let name = Path::new(&upload.name);
    let contents = &upload.contents;

    let cow_name;
    let name = if let Some(name) = name.file_name() {
        cow_name = name.to_string_lossy();
        &*cow_name
    } else {
        return Ok(Flash::error(Redirect::to("/"), "Plik nie ma właściwej nazwy."));
    };

    let upload_path = Path::new("uploads").join(user_id.0.to_string());

    fs::create_dir_all(&upload_path)?;
    File::create(upload_path.join(name))?.write(contents)?;

    diesel::insert(&NewFile {
                        name: name,
                        user_id: user_id.0,
                    }).into(files::table)
            .execute(&*connection)?;

    Ok(Flash::success(Redirect::to("/"), "Plik został wrzucony."))
}

pub fn routes() -> Vec<Route> {
    routes![upload]
}
