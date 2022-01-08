use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub enum MyState {
    LoggedIn {
        my_games: Vec<String>,
    },
    LoggedOut,
}
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum GameInfo {
    WaitingForPlayers { players: Vec<Player>, you: Option<Player> },
    Game(Perspective),
}
#[derive(Serialize, Deserialize, Debug)]
pub enum GameCommand {
    JoinGame(Player),
    LeaveGame,
    StartGame,
    Command(Command),
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, enum_utils::FromStr, enum_utils::IterVariants)]
pub enum Player {
    Gundla,
    Sarah,
    Marie,
    Zacharias,
}
impl Player {
    pub fn all() -> impl Iterator<Item = Player> + Clone {
        Self::iter()
    }
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Perspective {
    pub you: PlayerState,
    pub your_player_index: usize,
    pub players: Vec<PerspectivePlayer>,

    pub item_stack: usize,
    pub turn: PerspectiveTurnState,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PerspectivePlayer {
    pub player: Player,
    pub job: Option<Job>,
    pub item_count: usize,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum PerspectiveTurnState {
    TurnStart { player: Player },
    GameOver { winner: Faction },
    TradePending { offerer: Player, target: Player, item: Option<Item> },
    ResolvingTradeTrigger { offerer: Player, target: Player, is_first_item: bool, trigger: PerspectiveTradeTriggerState }, // for sextant, item selections are cleared
    Attacking { attacker: Player, defender: Player, state: PerspectiveAttackState }, // AttackState info ís always public
    DonatingItem { donor: Player },
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum PerspectiveAttackState {
    Normal(AttackState),
    FinishResolvingCredentials { target_faction: Faction, target_job: Job },
    FinishResolvingItems { target_items: Vec<Item> },
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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
    DonatingItem { donor: Player, followup: ItemDonationFollowup },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ItemDonationFollowup {
    NextPlayer(Player),
    TradeTriggers {
        offerer: Player,
        target: Player,
        item: Item,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum AttackState {
    WaitingForPriest { passed: HashSet<Player> },
    PayingPriest { priest: Player },
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

pub type BuffScore = i8;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Buff {
    pub user: Player,
    pub source: BuffSource,
    pub raw_score: BuffScore, // Raw score is twice actual strength and 1 if it breaks ties
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
impl AttackSupport {
    pub fn vote_value(&self) -> BuffScore {
        match self {
            AttackSupport::Attack => 1,
            AttackSupport::Defend => -1,
            AttackSupport::Abstain => 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum PerspectiveTradeTriggerState {
    Priviledge { items: Option<Vec<Item>> },
    Monocle { faction: Option<Faction> },
    Coat { available_jobs: Option<Vec<Job>> },
    Sextant { item_selections: HashMap<Player, Item>, is_forward: Option<bool> },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TradeTriggerState {
    Priviledge,
    Monocle,
    Coat,
    Sextant { item_selections: HashMap<Player, Item>, is_forward: Option<bool> },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Command {
    Pass,
    AnnounceVictory { teammates: Vec<Player> },

    OfferTrade { target: Player, item: Item },
    RejectTrade,
    AcceptTrade { item: Item },
    PickNewJob { job: Job },
    SelectSextantItem { item: Item },
    SetSextantDirection { forward: bool },

    InitiateAttack { player: Player },
    UsePriest { priest: bool },
    PayPriest { item: Item },
    DeclareSupport { support: AttackSupport },
    Hypnotize { target: Option<Player> },
    ItemOrJob {
        buff: Option<BuffSource>, // None is passing
        target: Option<Player> // Only for poison mixer atm, will have additional uses in expansion
    },
    ClaimReward { steal_items: bool },
    StealItem { item: Item, give_back: Option<Item> },

    DonateItem { target: Player, item: Item },
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
    pub fn raw_score(&self, user_type: AttackRole) -> Option<BuffScore> {
        match (self, user_type) {
            (BuffSource::Item(Item::Dagger), AttackRole::Attacker) => Some(2),
            (BuffSource::Job(Job::Thug), AttackRole::Attacker) => Some(2),
            (BuffSource::Item(Item::Gloves), AttackRole::Defender) => Some(-2),
            (BuffSource::Job(Job::GrandMaster), AttackRole::Defender) => Some(-2),
            (BuffSource::Item(Item::PoisonRing), AttackRole::Defender) => Some(-1),
            (BuffSource::Item(Item::PoisonRing), AttackRole::Attacker) => Some(1),
            (BuffSource::Job(Job::Duelist), AttackRole::Attacker) => Some(2),
            (BuffSource::Job(Job::Duelist), AttackRole::Defender) => Some(-2),
            (BuffSource::Item(Item::CastingKnives), AttackRole::AttackSupport(AttackSupport::Attack)) => Some(2),
            (BuffSource::Item(Item::Whip), AttackRole::AttackSupport(AttackSupport::Defend)) => Some(-2),
            (BuffSource::Job(Job::Bodyguard), AttackRole::AttackSupport(AttackSupport::Attack)) => Some(2),
            (BuffSource::Job(Job::Bodyguard), AttackRole::AttackSupport(AttackSupport::Defend)) => Some(-2),
            _ => None
        }
    }
}


impl PlayerState {
    pub fn use_job(&mut self, job: Job) -> Result<(), JobUseError> {
        if self.job == job && !(self.job_is_visible && job.once()) {
            self.job_is_visible = true;
            Ok(())
        } else {
            Err(JobUseError)
        }
    }
}

impl Job {
    pub fn once(&self) -> bool {
        match self {
            Job::Clairvoyant | Job::Diplomat | Job::Doctor | Job::Duelist | Job::PoisonMixer | Job::Priest => true,
            _ => false
        }
    }
}

#[derive(Debug)]
pub struct JobUseError;

