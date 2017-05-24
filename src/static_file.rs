use std::io;
use std::path::{Path, PathBuf};

use rocket::response::NamedFile;

#[get("/<path..>", rank = 5)]
pub fn static_file(path: PathBuf) -> io::Result<NamedFile> {
    NamedFile::open(Path::new("static/").join(path))
}
