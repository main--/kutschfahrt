-- Schema v1
CREATE TABLE game_players (
    gameid text NOT NULL,
    steamid bigint NOT NULL,
    player_character text NOT NULL,

    PRIMARY KEY(gameid, steamid),
    UNIQUE(gameid, player_character)
);

CREATE TABLE game_state (
    gameid text NOT NULL PRIMARY KEY,
    state text NOT NULL
);

