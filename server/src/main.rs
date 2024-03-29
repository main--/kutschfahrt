use rocket::{State, Shutdown};

use rocket::fs::{NamedFile, FileServer};
use rocket::serde::json::Json;
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
use rocket::tokio::select;
use rocket::response::stream::{EventStream, Event};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use kutschfahrt::State as KutschfahrtState;
use web_protocol::{GameCommand, MyState, GameInfo};

mod login;
use login::LoggedIn;

mod error;
use error::{Result, Error};



#[rocket::get("/me", rank = 1)]
async fn me_loggedin(db: &State<SqlitePool>, l: LoggedIn) -> Result<Json<MyState>> {
    let my_games = sqlx::query_scalar!("SELECT gameid FROM game_players WHERE steamid = ?", l.steamid).fetch_all(&**db).await?;
    Ok(Json(MyState::LoggedIn { my_games }))
}
#[rocket::get("/me", rank = 2)]
fn me_loggedout() -> Json<MyState> { Json(MyState::LoggedOut) }


#[rocket::get("/game/<id>")]
async fn game_get(db: &State<SqlitePool>, id: String, l: LoggedIn) -> Result<Json<GameInfo>> {
    let state = sqlx::query_scalar!("SELECT state FROM game_state WHERE gameid = ?", id).fetch_optional(&**db).await?;
    let you = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ? AND steamid = ?", id, l.steamid).fetch_optional(&**db).await?;
    let you = you.and_then(|x| x.parse().ok());
    Ok(Json(match (state, you) {
        (None, you) => {
            let players = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ?", id).fetch_all(&**db).await?;
            let players = players.into_iter().map(|x| x.parse().unwrap()).collect();
            GameInfo::WaitingForPlayers { players, you }
        }
        (Some(s), Some(you)) => {
            let state: KutschfahrtState = serde_json::from_str(&s)?;
            GameInfo::Game(state.perspective(you))
        }
        (Some(_), None) => unimplemented!("spectator mode"),
    }))
}

#[rocket::get("/game/<id>/events")]
async fn game_events(db: &State<SqlitePool>, id: String, l: LoggedIn, queue: &State<Sender<String>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    let db = (*db).clone();
    EventStream! {
        'outer: loop {
            let state = sqlx::query_scalar!("SELECT state FROM game_state WHERE gameid = ?", id).fetch_optional(&db).await.unwrap();
            let you = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ? AND steamid = ?", id, l.steamid).fetch_optional(&db).await.unwrap();
            let you = you.and_then(|x| x.parse().ok());
            let msg = match (state, you) {
                (None, you) => {
                    let players = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ?", id).fetch_all(&db).await.unwrap();
                    let players = players.into_iter().map(|x| x.parse().unwrap()).collect();
                    GameInfo::WaitingForPlayers { players, you }
                }
                (Some(s), Some(you)) => {
                    let state: KutschfahrtState = serde_json::from_str(&s).unwrap();
                    GameInfo::Game(state.perspective(you))
                }
                (Some(_), None) => unimplemented!("spectator mode"),
            };

            yield Event::json(&msg);

            // wait for updates to this gameid
            loop {
                select! {
                    msg = rx.recv() => match msg {
                        Ok(i) if i == id => break,
                        Ok(_) => (),
                        Err(RecvError::Closed) => break 'outer,
                        Err(RecvError::Lagged(_)) => continue 'outer,
                    },
                    _ = &mut end => break 'outer,
                };
            }
        }
    }
}

#[rocket::post("/game/<id>", data = "<cmd>")]
async fn game_post(cmd: Json<GameCommand>, db: &State<SqlitePool>, id: String, l: LoggedIn, queue: &State<Sender<String>>) -> Result<()> {
    let state = sqlx::query_scalar!("SELECT state FROM game_state WHERE gameid = ?", id).fetch_optional(&**db).await?;
    match (cmd.into_inner(), state) {
        (GameCommand::JoinGame(player), None) => {
            let player = player.to_string();
            sqlx::query!("INSERT INTO game_players(gameid, steamid, player_character) VALUES (?, ?, ?)", id, l.steamid, player).execute(&**db).await?;
        }
        (GameCommand::LeaveGame, None) => {
            sqlx::query!("DELETE FROM game_players WHERE gameid = ? AND steamid = ?", id, l.steamid).execute(&**db).await?;
        }
        (GameCommand::StartGame, None) => {
            let players = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ?", id).fetch_all(&**db).await?;
            let players = players.into_iter().map(|x| x.parse().unwrap()).collect();

            let state = KutschfahrtState::new(players, &mut rand::thread_rng());

            let state = serde_json::to_string(&state)?;
            sqlx::query!("INSERT INTO game_state(gameid, state) VALUES (?, ?)", id, state).execute(&**db).await?;
        }
        (GameCommand::Command(c), Some(s)) => {
            let you = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ? AND steamid = ?", id, l.steamid).fetch_one(&**db).await?;
            let you = you.parse().unwrap();
            let mut state: KutschfahrtState = serde_json::from_str(&s)?;
            state.apply_command(you, c)?;
            let state = serde_json::to_string(&state)?;
            sqlx::query!("UPDATE game_state SET state = ? WHERE gameid = ?", state, id).execute(&**db).await?;
        }
        _ => return Err(Error::CommandDoesNotMatchGameState),
    }
    let _ = queue.send(id);
    Ok(())
}

#[rocket::get("/<_path..>", rank = 100)]
async fn spa_fallback(_path: std::path::PathBuf) -> NamedFile {
    NamedFile::open("../client/dist/index.html").await.unwrap()
}

async fn create_db_pool() -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        //.max_connections(8)
        .connect("sqlite:kutschfahrt.sqlite").await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

#[rocket::launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(create_db_pool().await.unwrap())
        .manage(channel::<String>(1024).0)
        .mount("/", FileServer::from("../client/dist"))
        .mount("/", rocket::routes![spa_fallback])
        .mount("/api/", rocket::routes![

            login::login,
            login::login_cb,
            login::logout,
            game_get,
            game_events,
            game_post,
            me_loggedin,
            me_loggedout,
        ])
}

