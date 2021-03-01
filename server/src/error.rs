use std::io::Cursor;
use kutschfahrt::CommandError;
use thiserror::Error;
use rocket::{Request, Response};
use rocket::response::{Responder, self};
use rocket::http::{Status, ContentType};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    DatabaseMigration(#[from] sqlx::migrate::MigrateError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid game command: {0}")]
    Command(#[from] CommandError),
    #[error("Command does not match game state")]
    CommandDoesNotMatchGameState,
}
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        eprintln!("{:?}", self);
        let response_string = self.to_string();
        Response::build()
            .sized_body(response_string.len(), Cursor::new(response_string))
            .header(ContentType::new("text", "plain"))
            .status(Status::InternalServerError)
            .ok()
    }
}

pub type Result<T> = std::result::Result<T, Error>;

