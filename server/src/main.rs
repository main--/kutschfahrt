use rocket_contrib::json::Json;
use rocket::State;
use serde_derive::{Serialize, Deserialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use kutschfahrt::{Player, Command, State as KutschfahrtState, Perspective};

mod login;
use login::LoggedIn;

mod error;
use error::{Result, Error};



#[derive(Serialize)]
enum GameInfo {
    WaitingForPlayers { players: Vec<Player>, you: Option<Player> },
    Game(Perspective),
}

#[rocket::get("/game/<id>")]
async fn game_get<'a>(db: State<'a, SqlitePool>, id: String, l: LoggedIn) -> Result<Json<GameInfo>> {
    let state = sqlx::query_scalar!("SELECT state FROM game_state WHERE gameid = ?", id).fetch_optional(&*db).await?;
    let you = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ? AND steamid = ?", id, l.steamid).fetch_optional(&*db).await?;
    let you = you.and_then(|x| x.parse().ok());
    Ok(Json(match (state, you) {
        (None, you) => {
            let players = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ?", id).fetch_all(&*db).await?;
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

#[derive(Deserialize)]
enum GameCommand {
    JoinGame(Player),
    LeaveGame,
    StartGame,
    Command(Command),
}

#[rocket::post("/game/<id>", data = "<cmd>")]
async fn game_post<'a>(cmd: Json<GameCommand>, db: State<'a, SqlitePool>, id: String, l: LoggedIn) -> Result<()> {
    let state = sqlx::query_scalar!("SELECT state FROM game_state WHERE gameid = ?", id).fetch_optional(&*db).await?;
    match (cmd.into_inner(), state) {
        (GameCommand::JoinGame(player), None) => {
            let player = player.to_string();
            sqlx::query!("INSERT INTO game_players(gameid, steamid, player_character) VALUES (?, ?, ?)", id, l.steamid, player).execute(&*db).await?;
        }
        (GameCommand::LeaveGame, None) => {
            sqlx::query!("DELETE FROM game_players WHERE gameid = ? AND steamid = ?", id, l.steamid).execute(&*db).await?;
        }
        (GameCommand::StartGame, None) => {
            let players = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ?", id).fetch_all(&*db).await?;
            let players = players.into_iter().map(|x| x.parse().unwrap()).collect();

            let mut players: Vec<_> = players;
            players.push(Player::Zacharias);
            players.push(Player::Sarah);

            let state = KutschfahrtState::new(players, &mut rand::thread_rng());

            let state = serde_json::to_string(&state)?;
            sqlx::query!("INSERT INTO game_state(gameid, state) VALUES (?, ?)", id, state).execute(&*db).await?;
        }
        (GameCommand::Command(c), Some(s)) => {
            let you = sqlx::query_scalar!("SELECT player_character FROM game_players WHERE gameid = ? AND steamid = ?", id, l.steamid).fetch_one(&*db).await?;
            let you = you.parse().unwrap();
            let mut state: KutschfahrtState = serde_json::from_str(&s)?;
            state.apply_command(you, c)?;
            let state = serde_json::to_string(&state)?;
            sqlx::query!("UPDATE game_state SET state = ? WHERE gameid = ?", state, id).execute(&*db).await?;
        }
        _ => return Err(Error::CommandDoesNotMatchGameState),
    }
    Ok(())
}


async fn create_db_pool() -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        //.max_connections(8)
        .connect("sqlite::memory:").await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

#[rocket::launch]
async fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .manage(create_db_pool().await.unwrap())
        //.mount("/", StaticFiles::from("static/"))
        .mount("/", rocket::routes![login::login, game_get, game_post])
}

