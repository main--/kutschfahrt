use rocket::response::Redirect;
use rocket::http::uri::Origin;
use rocket::http::{CookieJar, Cookie, SameSite};
use rocket::request::{Outcome, FromRequest};
use steam_auth::{Verifier, Redirector};

pub struct LoggedIn {
    pub steamid: i64,
}

const USERID: &str = "userid";
#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for LoggedIn {
    type Error = ();
    
    async fn from_request(req: &'a rocket::Request<'r>) -> Outcome<Self, Self::Error> {
        let jar = <&'a CookieJar>::from_request(req).await.unwrap();
        let steamid = match jar.get_private(USERID).and_then(|x| x.value().parse().ok()) {
            Some(x) => x,
            None => return Outcome::Forward(()),
        };
        Outcome::Success(LoggedIn { steamid })
    }
}


#[rocket::get("/login?<returnurl>")]
pub async fn login<'a>(returnurl: String) -> Redirect {
    Redirect::to(Redirector::new(returnurl, rocket::uri!(login_cb).to_string()).unwrap().url().to_string())
}
#[rocket::get("/login_callback")]
pub async fn login_cb<'a>(cookies: &'a CookieJar<'a>, qs: &'a Origin<'a>) -> Redirect {
    if let Some(q) = qs.query() {
        // TODO: rework the error handling here
        let (req, verifier) = Verifier::from_querystring(q).unwrap();
        let (parts, body) = req.into_parts();
        let resp = reqwest::Client::new()
            .post(&parts.uri.to_string())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await.unwrap();

        match verifier.verify_response(resp.text().await.unwrap()) {
            Ok(steam_id) => {
                let mut c = Cookie::new(USERID, steam_id.to_string());
                c.set_same_site(SameSite::Lax);
                cookies.add_private(c);
            }
            Err(e) => eprintln!("There was an error authenticating: {}", e),
        }
    }
    Redirect::to("/")
}

#[rocket::get("/logout")]
pub fn logout(cookies: &CookieJar) -> Redirect {
    cookies.remove_private(Cookie::named(USERID));
    Redirect::to("/")
}

