use std::collections::{HashMap, HashSet};
use std::fmt::Display;
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
    /// when a game has already started and you're not part of it
    Spectating(SpectatorPerspective),
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
    Marie,
    Gundla,
    Sarah,
    Romana,
    Theodora,
    Basilius,
    Juan,
    Zacharias,
    Michel,
    Sinclair,
}
impl Player {
    pub fn all() -> impl Iterator<Item = Player> + Clone {
        Self::iter()
    }
}
impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Player::Marie => "Marie Sauniére",
            Player::Gundla => "Gundla von Hochberg",
            Player::Sarah => "Sarah Mac Mullin",
            Player::Romana => "Romana Baranov",
            Player::Theodora => "Theodora Krayenborg",
            Player::Basilius => "Basilius Kartov",
            Player::Juan => "Juan Tirador",
            Player::Zacharias => "Bruder Zacharias",
            Player::Michel => "Michel de Molay",
            Player::Sinclair => "Sir Henry Sinclair",
        };
        write!(f, "{}", name)
    }
}
// https://boardgamegeek.com/image/301727/die-kutschfahrt-zur-teufelsburg

/// A perspective is like `State` but contanis only information
/// that a particular player is allowed to see.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Perspective {
    pub you: PlayerState,
    pub your_player_index: usize,
    pub players: Vec<PerspectivePlayer>,

    pub item_stack: usize,
    pub action_log: Vec<ActionLogEntry>,
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
    TurnEndPhase { player: Player },
    DoingClairvoyant { player: Player, item_stack: Option<Vec<Item>> },
    UnsuccessfulDiplomat { diplomat: Player, target: Player, inventory: Option<Vec<Item>> },

    GameOver { winner: WinningFaction },
    TradePending { offerer: Player, target: Player, item: Option<Item> },
    ResolvingTradeTrigger { giver: Player, receiver: Player, trigger: PerspectiveTradeTriggerState }, // for sextant, item selections are cleared
    Attacking { attacker: Player, defender: Player, state: PerspectiveAttackState }, // AttackState info ís always public
    DonatingItem { donor: Player },
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum PerspectiveAttackState {
    Normal(AttackState),
    FinishResolvingNeedFactionIndex,
    FinishResolvingCredentials { target_faction: Faction, target_job: Job },
    FinishResolvingItems { target_items: Vec<Item> },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum FactionKind {
    Normal(Faction),
    ThreePlayer([Faction; 3]),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PlayerState {
    pub faction: FactionKind,
    pub job: Job,
    pub job_is_visible: bool,
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SpectatorPerspective {
    pub players: Vec<PerspectivePlayer>,

    pub item_stack: usize,
    pub action_log: Vec<ActionLogEntry>,
    pub turn: PerspectiveTurnState,
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
impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Item::Key => "Key",
            Item::Goblet => "Goblet",
            Item::BagKey => "Bag (Key)",
            Item::BagGoblet => "Bag (Goblet)",
            Item::BlackPearl => "Black Pearl",
            Item::Dagger => "Dagger",
            Item::Gloves => "Gloves",
            Item::PoisonRing => "Poison Ring",
            Item::CastingKnives => "Casting Knives",
            Item::Whip => "Whip",
            Item::Priviledge => "Priviledge",
            Item::Monocle => "Monocle",
            Item::BrokenMirror => "Broken Mirror",
            Item::Sextant => "Sextant",
            Item::Coat => "Coat",
            Item::Tome => "Tome",
            Item::CoatOfArmorOfTheLoge => "Coat of Armor of the Loge",
        })
    }
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
impl std::fmt::Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Job::Thug => "Thug",
            Job::GrandMaster => "Grandmaster",
            Job::Bodyguard => "Bodyguard",
            Job::Duelist => "Duelist",
            Job::PoisonMixer => "Poison Mixer",
            Job::Doctor => "Doctor",
            Job::Priest => "Priest",
            Job::Hypnotist => "Hypnotist",
            Job::Diplomat => "Diplomat",
            Job::Clairvoyant => "Clairvoyant",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum WinningFaction {
    Normal(Faction),
    /// Loge
    Traitor(Player),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Faction {
    Order,
    Brotherhood,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TurnState {
    WaitingForQuickblink(Player),
    /// for end-of-turn clairvoyant and diplomat
    WaitingForEndTurn(Player),

    // job actions are currently only possible at turn start
    DoingClairvoyant { clairvoyant: Player, next: Player },
    UnsuccessfulDiplomat { diplomat: Player, target: Player },

    GameOver { winner: WinningFaction },
    TradePending {
        offerer: Player,
        target: Player,
        item: Item,
    },
    ResolvingTradeTrigger {
        giver: Player,
        receiver: Player,
        trigger: TradeTriggerState,
        next_state: FollowupState,
    },
    Attacking {
        attacker: Player,
        defender: Player,
        state: AttackState,
    },
    DonatingItem { donor: Player, followup: FollowupState },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum FollowupState {
    State(Box<TurnState>),
    TradeTriggers {
        giver: Player,
        receiver: Player,
        item: Item,
        next_state: Box<TurnState>,
    },
}
impl FollowupState {
    pub fn end_phase(player: Player) -> Self {
        FollowupState::State(Box::new(TurnState::WaitingForEndTurn(player)))
    }
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
        three_player_faction_index: Option<usize>,
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
            AttackSupport::Attack => 2,
            AttackSupport::Defend => -2,
            AttackSupport::Abstain => 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum PerspectiveTradeTriggerState {
    Priviledge { items: Option<Vec<Item>> },
    Monocle { faction: Option<Faction>, three_player_faction_index: Option<usize> },
    Coat { available_jobs: Option<Vec<Job>> },
    Sextant { item_selections: HashMap<Player, Item>, is_forward: Option<bool> },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TradeTriggerState {
    Priviledge,
    Monocle { three_player_faction_index: Option<usize> },
    Coat,
    Sextant { item_selections: HashMap<Player, Item>, is_forward: Option<bool> },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Command {
    Pass,
    AnnounceVictory { flavor: VictoryFlavor },

    UseDiplomat { target: Player, item: Item, return_item: Item },
    UseClairvoyant,
    ClairvoyantSetItems {
        /// always exactly two items unless the stack has fewer than two items in total
        top_items: Vec<Item>
    },

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

    ThreePlayerSelectFactionIndex { index: usize },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VictoryFlavor {
    Normal { teammates: Vec<Player> },
    Loge,
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
    pub fn effective_faction(&self) -> Faction {
        match &self.faction {
            FactionKind::Normal(faction) => *faction,
            FactionKind::ThreePlayer(factions) => {
                if factions.iter().filter(|&&x| x == Faction::Order).count() >= 2 {
                    Faction::Order
                } else {
                    Faction::Brotherhood
                }
            }
        }
    }

    pub fn faction_by_index(&self, idx: Option<usize>) -> Option<Faction> {
        match (&self.faction, idx) {
            (&FactionKind::Normal(faction), None) => Some(faction),
            (&FactionKind::Normal(_), Some(_)) => panic!("Faction index specified but this is not a 3-player game"),
            (&FactionKind::ThreePlayer(_), None) => None,
            (&FactionKind::ThreePlayer(factions), Some(i)) => Some(factions[i]),
        }
    }

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

pub fn inventory_limit(players: usize) -> usize {
    match players {
        i if i < 3 => panic!("invalid player count"),
        3 => 8,
        4 => 6,
        _ => 5,
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ActionLogEntry {
    Pass { actor: Player },
    AnnounceVictory { actor: Player },

    UseDiplomat { actor: Player, target: Player, item: Item, success: bool },
    UseClairvoyant { actor: Player },

    TradeOffer { offerer: Player, target: Player, accepted: bool },
    Attack { attacker: Player, target: Player },

    TradeTrigger { giver: Player, receiver: Player, item: Item },
    DonateItem { giver: Player, receiver: Player },
}
