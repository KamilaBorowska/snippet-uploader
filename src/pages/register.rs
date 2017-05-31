use rocket::Route;
use rocket::http::Session;
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket_contrib::Template;

use diesel;
use diesel::result::DatabaseErrorKind;
use db::Connection;

use db::user::{self, RegisterForm};

use Context;

#[post("/", data = "<form>")]
fn register(mut session: Session,
            form: Form<RegisterForm>,
            db: Connection)
            -> user::Result<Result<Flash<Redirect>, Template>> {
    let user = form.get();
    match user.register(&db) {
        Ok(id) => {
            id.login(&mut session)?;
            Ok(Ok(Flash::success(Redirect::to("/"), "Konto zarejestrowane.")))
        }
        Err(e) => {
            let error = match *e.kind() {
                user::ErrorKind::PasswordsNotIdentical => "Hasła nie są identyczne",
                user::ErrorKind::PasswordTooShort => "Hasło za krótkie",
                user::ErrorKind::Query(diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => "Użytkownik już istnieje",
                user::ErrorKind::Query(diesel::result::Error::DatabaseError(_, _)) => "Użytkownik musi składać się tylko z liter",
                _ => return Err(e),
            };
            Ok(Err(page_internal(Some(Flash::error((), error)), &user.name)))
        }
    }
}

fn page_internal(flash: Option<FlashMessage>, name: &str) -> Template {
    let message = flash.as_ref().map(|f| f.msg());
    Template::render("register",
                     &Context {
                          title: "Rejestracja",
                          flash: message,
                          page: name,
                      })
}

#[get("/")]
fn page() -> Template {
    page_internal(None, "")
}

pub fn routes() -> Vec<Route> {
    routes![register, page]
}
