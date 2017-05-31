use rocket::{Request, Data, Outcome};
use rocket::data::{self, FromData};
use rocket::http::Status;
use multipart::server::Multipart;

use std::error::Error;
use std::io::{Cursor, Read};

#[derive(Debug)]
pub struct FileUpload {
    pub name: String,
    pub contents: Vec<u8>,
}

impl FromData for FileUpload {
    type Error = String;

    fn from_data(request: &Request, data: Data) -> data::Outcome<FileUpload, String> {
        let ct = match request.headers().get_one("Content-Type") {
            Some(ct) => ct,
            None => {
                return Outcome::Failure((Status::UnprocessableEntity,
                                         "Expected Content-Type".into()))
            }
        };
        let boundary_str = "boundary=";
        let idx = match ct.find(boundary_str) {
            Some(idx) => idx,
            None => {
                return Outcome::Failure((Status::UnprocessableEntity, "Expected boundary".into()))
            }
        };
        let boundary = &ct[idx + boundary_str.len()..];

        let mut out = Vec::new();
        if let Err(e) = data.stream_to(&mut out) {
            return Outcome::Failure((Status::UnprocessableEntity, String::from(e.description())));
        }

        let mut mp = Multipart::with_body(Cursor::new(out), boundary);

        let mut outcome = Outcome::Failure((Status::UnprocessableEntity, "File not found".into()));

        let _ =
            mp.foreach_entry(|mut entry| if entry.name.as_str() == "file" {
                                 let mut data = Vec::new();
                                 let upload = entry
                                     .data
                                     .as_file()
                                     .ok_or(String::from("File doesn't exist"))
                                     .and_then(|f| {
                    let _ = f.read_to_end(&mut data);
                    Ok(FileUpload {
                           name: f.filename
                               .as_ref()
                               .map(String::as_str)
                               .unwrap_or("unnamed")
                               .into(),
                           contents: data,
                       })
                });
                                 outcome = match upload {
                                     Ok(out) => Outcome::Success(out),
                                     Err(e) => Outcome::Failure((Status::UnprocessableEntity, e)),
                                 }
                             });

        outcome
    }
}
