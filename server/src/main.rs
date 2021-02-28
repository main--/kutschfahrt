use rocket_contrib::json::Json;
use rocket::State;
use serde_derive::Serialize;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use anyhow::Result;
use rocket::response::Redirect;
use rocket::futures::TryStreamExt;

mod login;
use login::LoggedIn;


#[derive(Serialize)]
struct Stonks {
    foo: i32,
    bar: String,
}

type RocketRes<T> = std::result::Result<T, rocket::response::Debug<sqlx::Error>>;

#[rocket::get("/")]
async fn hello<'a>(s: State<'a, SqlitePool>) -> RocketRes<Json<Stonks>> {
    let mut rows = sqlx::query!("SELECT * from stonks limit 2").fetch(&*s);
    while let Some(row) = rows.try_next().await? {
        println!("{:?}", row);
    }
    Ok(Json(Stonks {
        foo: 42,
        bar: "memes".to_owned(),
    }))
}

#[rocket::get("/stonks")]
async fn stonks<'a>(l: LoggedIn) -> String {
    format!("stonks: {}", l.steamid)
}


async fn create_pool() -> Result<SqlitePool> {
    //let m = Migrator::new(std::path::Path::new("./migrations")).await?;
    
    let pool = SqlitePoolOptions::new()
        //.max_connections(8)
        .connect("sqlite::memory:").await?;
    sqlx::migrate!().run(&pool).await?;
    //m.run(&pool).await?;
    Ok(pool)
}


#[rocket::launch]
async fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .manage(create_pool().await.unwrap())
        //.mount("/", StaticFiles::from("static/"))
        .mount("/", rocket::routes![hello, login::login, stonks])
}

