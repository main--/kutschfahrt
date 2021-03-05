use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub enum MyState {
    LoggedIn {
        my_games: Vec<String>,
    },
    LoggedOut,
}
#[derive(Serialize, Deserialize)]
pub enum GameInfo {
    WaitingForPlayers { players: Vec<Player>, you: Option<Player> },
    Game(Perspective),
}
#[derive(Serialize, Deserialize)]
pub enum GameCommand {
    JoinGame(Player),
    LeaveGame,
    StartGame,
    Command(Command),
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, enum_utils::FromStr)]
pub enum Player {
    Gundla,
    Sarah,
    Marie,
    Zacharias,
}
impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
/*
Marie Sauniére
Gundla von Hochberg
random green bitch
Romana Baranov
Theodora Krayenborg
Basilius Kartov
Juan Tirador
Bruder Zacharias
Michel de Molay
Sir Henry Sinclair
*/

/// A perspective is like `State` but contanis only information
/// that a particular player is allowed to see.
#[derive(Debug, Serialize, Deserialize)]
pub struct Perspective {
    pub you: PlayerState,
    pub your_player_index: usize,
    pub players: Vec<PerspectivePlayer>,

    pub item_stack: usize,
    pub turn: PerspectiveTurnState,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PerspectivePlayer {
    pub player: Player,
    pub job: Option<Job>,
    pub item_count: usize,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum PerspectiveTurnState {
    TurnStart { player: Player },
    GameOver { winner: Faction },
    TradePending { offerer: Player, target: Player, item: Option<Item> },
    ResolvingTradeTrigger { offerer: Player, target: Player, trigger: TradeTriggerState }, // for sextant, item selections are cleared
    Attacking { attacker: Player, defender: Player, state: AttackState }, // AttackState info ís always public
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerState {
    pub faction: Faction,
    pub job: Job,
    pub job_is_visible: bool,
    pub items: Vec<Item>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Item {
    Key,
    Goblet,
    BagKey, // trigger: bag
    BagGoblet, // trigger: bag
    BlackPearl, 
    Dagger,
    Gloves,
    PoisonRing,
    CastingKnives,
    Whip, // Insert Zasa whip noises
    Priviledge, // trigger: view items
    Monocle, // trigger: view association
    BrokenMirror,
    Sextant, // trigger: xD
    Coat, // trigger: exchange occupation
    Tome, // trigger: trade occupation
    CoatOfArmorOfTheLoge
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Job {
    Thug,
    GrandMaster,
    Bodyguard,
    Duelist,
    PoisonMixer,
    Doctor,
    Priest,
    Hypnotist,
    Diplomat,
    Clairvoyant,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Faction {
    Order,
    Brotherhood,
    //Traitor,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TurnState {
    WaitingForQuickblink(Player),
    GameOver { winner: Faction },
    TradePending {
        offerer: Player,
        target: Player,
        item: Item,
    },
    ResolvingTradeTrigger {
        offerer: Player,
        target: Player,
        next_item: Option<Item>,
        trigger: TradeTriggerState,
    },
    Attacking {
        attacker: Player,
        defender: Player,
        state: AttackState,
    },
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum AttackState {
    WaitingForPriest,
    DeclaringSupport(HashMap<Player, AttackSupport>),
    WaitingForHypnotizer(HashMap<Player, AttackSupport>),
    ItemsOrJobs {
        votes: HashMap<Player, AttackSupport>,
        passed: HashSet<Player>,
        buffs: Vec<Buff>,
    },
    Resolving {
        winner: AttackWinner,
    },
    FinishResolving {
        winner: AttackWinner,
        steal_items: bool,
    },
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum AttackWinner {
    Attacker,
    Defender,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Buff {
    pub user: Player,
    pub source: BuffSource,
    pub value: i8, //Let's see if future Kutschfahrt Expansions make scores over 127 possible (currently capped at ~25)
    pub breaks_tie: i8 //This should have a sign too, so we can't use bool
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum BuffSource  {
    Item(Item),
    Job(Job),
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttackSupport {
    Attack,
    Defend,
    Abstain,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TradeTriggerState {
    Priviledge,
    Monocle,
    Coat,
    Sextant { item_selections: HashMap<Player, Item> },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Command {
    Pass,
    AnnounceVictory { teammates: Vec<Player> },

    OfferTrade { target: Player, item: Item },
    RejectTrade,
    AcceptTrade { item: Item },
    PickNewJob { job: Job },
    SelectSextantItem { item: Item },

    InitiateAttack { player: Player },
    UsePriest { priest: bool },
    DeclareSupport { support: AttackSupport },
    Hypnotize { target: Option<Player> },
    ItemOrJob { buff: Option<BuffSource> }, // None is passing
    ClaimReward { steal_items: bool },
    StealItem { item: Item, give_back: Option<Item> },

    DoneLookingAtThings,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttackRole {
    Attacker,
    Defender,
    AttackSupport(AttackSupport)
}

impl BuffSource {
    pub fn legal(&self, user_type: AttackRole) -> bool {
        match user_type {
            AttackRole::Attacker => {
                match self {
                    BuffSource::Item(Item::Dagger) | BuffSource::Item(Item::PoisonRing) | BuffSource::Job(Job::Thug) => true,
                    _ => false
                }
            }
            AttackRole::Defender => {
                match self {
                    BuffSource::Item(Item::Gloves) | BuffSource::Item(Item::PoisonRing) | BuffSource::Job(Job::GrandMaster) => true,
                    _ => false
                }
            }
            AttackRole::AttackSupport(AttackSupport::Attack) => {
                match self {
                    BuffSource::Item(Item::CastingKnives) | BuffSource::Job(Job::Bodyguard) => true,
                    _ => false
                }
            }
            AttackRole::AttackSupport(AttackSupport::Defend) => {
                match self {
                    BuffSource::Item(Item::Whip) | BuffSource::Job(Job::Bodyguard) => true,
                    _ => false
                }
            }
            AttackRole::AttackSupport(AttackSupport::Abstain) => false
        }
    }

    pub fn score(&self) -> i8 {
        match self {
            BuffSource::Item(Item::PoisonRing) => 0,
            _ => 1, // yeah sorry that's not exciting. We have no fancy expansion stuff yet
        }
    }

    pub fn breaks_tie(&self) -> bool {
        match self {
            BuffSource::Item(Item::PoisonRing) => true,
            _ => false,
        }
    }
}

impl AttackRole {
    pub fn sign(&self) -> i8 {
        match self {
            AttackRole::Attacker | AttackRole::AttackSupport(AttackSupport::Attack) => 1,
            AttackRole::Defender | AttackRole::AttackSupport(AttackSupport::Defend) => -1,
            _ => 0
        }
    }
}
