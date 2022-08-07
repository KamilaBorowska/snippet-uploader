use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::request::FlashMessage;
use rocket::response::{Debug, Flash, Redirect};
use rocket::Route;
use rocket_dyn_templates::Template;

use crate::db::Connection;
use diesel;
use diesel::result::DatabaseErrorKind;

use crate::db::user::{self, RegisterForm};

use crate::Context;

#[post("/", data = "<form>")]
async fn register(
    cookie_jar: &CookieJar<'_>,
    form: Form<RegisterForm>,
    connection: Connection,
) -> Result<Result<Flash<Redirect>, Template>, Debug<user::Error>> {
    let user = form.into_inner();
    let (res, user) = connection.run(|c| (user.register(c), user)).await;
    match res {
        Ok(id) => {
            id.login(cookie_jar, connection).await?;
            Ok(Ok(Flash::success(
                Redirect::to("/"),
                "Konto zarejestrowane.",
            )))
        }
        Err(e) => {
            let error = match *e.kind() {
                user::ErrorKind::PasswordsNotIdentical => "Hasła nie są identyczne",
                user::ErrorKind::PasswordTooShort => "Hasło za krótkie",
                user::ErrorKind::Query(diesel::result::Error::DatabaseError(
                    DatabaseErrorKind::UniqueViolation,
                    _,
                )) => "Użytkownik już istnieje",
                user::ErrorKind::Query(diesel::result::Error::DatabaseError(_, _)) => {
                    "Użytkownik musi składać się tylko z liter"
                }
                _ => return Err(Debug(e)),
            };
            Ok(Err(page_internal(
                Some(Flash::error(cookie_jar, error)),
                &user.name,
            )))
        }
    }
}

fn page_internal(flash: Option<FlashMessage>, name: &str) -> Template {
    let message = flash.as_ref().map(|f| f.message());
    Template::render(
        "register",
        &Context {
            title: "Rejestracja",
            flash: message,
            page: name,
        },
    )
}

#[get("/")]
fn page() -> Template {
    page_internal(None, "")
}

pub fn routes() -> Vec<Route> {
    routes![register, page]
}
