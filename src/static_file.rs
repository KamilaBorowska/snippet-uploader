use std::io;
use std::path::{Path, PathBuf};

use rocket::fs::NamedFile;

#[get("/<path..>", rank = 5)]
pub async fn static_file(path: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).await
}
