#![allow(dead_code)]

use std::collections::{HashSet, HashMap};
use std::mem;
use std::iter;

use indexmap::IndexMap;
use serde_json;
use serde_derive::Serialize;
use rand::prelude::*;

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone, Copy)]
enum Player {
    Gundla,
    Sarah,
    Marie,
    Zacharias,
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

#[derive(Debug, Serialize)]
struct GameState {
    players: IndexMap<Player, PlayerState>,
    item_stack: Vec<Item>,
    job_stack: Vec<Job>,
}
#[derive(Debug, Serialize)]
struct State {
    game: GameState,
    turn: TurnState,
}
#[derive(Debug, Serialize)]
struct PlayerState {
    faction: Faction,
    job: Job,
    job_is_visible: bool,
    items: Vec<Item>,
}
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
enum Item {
    Key,
    Goblet,
    BagKey, // trigger: bag
    BagGoblet, // trigger: bag
    BlackPearl, 
    Dagger,
    Gloves,
    PoisonRing,
    CastingKnives,
    Whip,
    Priviledge, // trigger: view items
    Monocle, // trigger: view association
    BrokenMirror,
    Sextant, // trigger: xD
    Coat, // trigger: exchange occupation
    Tome, // trigger: trade occupation
    CoatOfArmorOfTheLoge
}
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
enum Job {
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
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
enum Faction {
    Order,
    Brotherhood,
    //Traitor,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
enum TurnState {
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

    Crashed,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
enum AttackState {
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
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
enum AttackWinner {
    Attacker,
    Defender,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
struct Buff {
    user: Player,
    source: BuffSource,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BuffSource  {
    Item(Item),
    Job(Job),
}
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum AttackSupport {
    Attack,
    Defend,
    Abstain,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
enum TradeTriggerState {
    Priviledge,
    Monocle,
    Coat,
    Sextant { item_selections: HashMap<Player, Item> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum Command {
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

#[derive(Debug)]
enum CommandError {
    NotYourTurn,
    InvalidCommandInThisContext,
    InvalidTargetPlayer,
    YouHaveAlreadyPassed,
    InvalidStealCommand,
}

impl GameState {
    fn next_player(&self, p: Player) -> Player {
        let index = self.players.get_index_of(&p).expect("Invalid player");
        let next_index = (index + 1) % self.players.len();
        *self.players.get_index(next_index).unwrap().0
    }
    fn attack_supporters<'a>(&'a self, attacker: Player, defender: Player) -> impl Iterator<Item=Player> + 'a {
        let keys = self.players.keys();
        let keys_twice = keys.clone().chain(keys);
        keys_twice.copied()
            .skip_while(move |&x| x != attacker)
            .take(self.players.len())
            .filter(move |&x| x != attacker && x != defender)
    }
}
impl State {
    fn apply_command(&mut self, actor: Player, c: Command) -> Result<(), CommandError> {
        let s = &mut self.game;
        self.turn = match mem::replace(&mut self.turn, TurnState::Crashed) {
            TurnState::Crashed => unreachable!(),
            TurnState::WaitingForQuickblink(p) => {
                if actor != p {
                    return Err(CommandError::NotYourTurn);
                }

                match c {
                    Command::Pass => {
                        TurnState::WaitingForQuickblink(s.next_player(p))
                    }
                    Command::AnnounceVictory { teammates } => {
                        unimplemented!();
                    }
                    Command::OfferTrade { target, item } => {
                        unimplemented!();
                    }
                    Command::InitiateAttack { player } => {
                        if !s.players.contains_key(&player) || actor == player {
                            return Err(CommandError::InvalidTargetPlayer);
                        }
                        TurnState::Attacking {
                            attacker: actor,
                            defender: player,
                            state: AttackState::WaitingForPriest,
                        }
                    }
                    _ => return Err(CommandError::InvalidCommandInThisContext),
                }
            }
            TurnState::Attacking { attacker, defender, state } => match state {
                AttackState::WaitingForPriest => {
                    if actor != defender {
                        return Err(CommandError::NotYourTurn);
                    }
                    match c {
                        Command::UsePriest { priest: true } => {
                            let defp = s.players.get(&defender).unwrap();
                            if (defp.job != Job::Priest) || defp.job_is_visible {
                                // TODO: better error maybe
                                return Err(CommandError::InvalidCommandInThisContext);
                            }

                            // TODO: resolve priest usage (needs new state I think)
                            unimplemented!();
                        },
                        Command::UsePriest { priest: false } => {
                            TurnState::Attacking { attacker, defender, state: AttackState::DeclaringSupport(HashMap::new()) }
                        },
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                }
                AttackState::DeclaringSupport(mut votes) => {
                    let next_voter = s.attack_supporters(attacker, defender).nth(votes.len()).unwrap();
                    if actor != next_voter {
                        return Err(CommandError::NotYourTurn);
                    }
                    match c {
                        Command::DeclareSupport { support } => {
                            votes.insert(actor, support);
                            if votes.len() == s.players.len() - 2 {
                                TurnState::Attacking { attacker, defender, state: AttackState::WaitingForHypnotizer(votes) }
                            } else {
                                TurnState::Attacking { attacker, defender, state: AttackState::DeclaringSupport(votes) }
                            }
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                }
                AttackState::WaitingForHypnotizer(mut votes) => {
                    if actor != attacker {
                        return Err(CommandError::NotYourTurn);
                    }
                    match c {
                        Command::Hypnotize { target } => {
                            if let Some(target) = target {
                                votes.insert(target, AttackSupport::Abstain);
                            }
                            TurnState::Attacking { attacker, defender, state: AttackState::ItemsOrJobs { votes, passed: HashSet::new(), buffs: Vec::new() }}
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                }
                AttackState::ItemsOrJobs { votes, mut passed, mut buffs } => {
                    if passed.contains(&actor) {
                        return Err(CommandError::YouHaveAlreadyPassed);
                    }
                    match c {
                        Command::ItemOrJob { buff: None } => {
                            passed.insert(actor);
                            if passed.len() == s.players.len() {
                                // TODO: actually resolve the fight here
                                unimplemented!();
                                //return Ok(TurnState::Attacking { attacker, defender, state: AttackState::Resolving })
                            }
                        }
                        Command::ItemOrJob { buff: Some(buff) } => {
                            // TODO: validate and apply buff
                            passed.clear();
                            unimplemented!();
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                    TurnState::Attacking { attacker, defender, state: AttackState::ItemsOrJobs { votes, passed, buffs } }
                }
                AttackState::Resolving { winner } => {
                    let winner_player = match winner {
                        AttackWinner::Attacker => attacker,
                        AttackWinner::Defender => defender,
                    };
                    if actor != winner_player {
                        return Err(CommandError::NotYourTurn);
                    }
                    match c {
                        Command::ClaimReward { steal_items } => {
                            TurnState::Attacking { attacker, defender, state: AttackState::FinishResolving { winner, steal_items } }
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                }
                AttackState::FinishResolving { winner, steal_items } => {
                    let winner_player = match winner {
                        AttackWinner::Attacker => attacker,
                        AttackWinner::Defender => defender,
                    };
                    if actor != winner_player {
                        return Err(CommandError::NotYourTurn);
                    }
                    match c {
                        Command::DoneLookingAtThings if !steal_items => (),
                        Command::StealItem { item, give_back } if steal_items => {
                            // borrowing is complicated sometimes :(
                            let attacker_del_idx;
                            let defender_del_idx;
                            {
                                let attacker_items = &s.players.get(&attacker).unwrap().items;
                                let defender_items = &s.players.get(&defender).unwrap().items;
                                if give_back.is_some() != (defender_items.len() == 1) {
                                    // give back an item exactly when defender has exactly 1 item
                                    return Err(CommandError::InvalidStealCommand)
                                }
                                // TODO: implement inventory limit and item donation
                                defender_del_idx = defender_items.iter().position(|x| *x == item).ok_or(CommandError::InvalidStealCommand)?;
                                attacker_del_idx = match give_back {
                                    None => None,
                                    Some(i) => Some(attacker_items.iter().position(|x| *x == i).ok_or(CommandError::InvalidStealCommand)?),
                                };
                            }
                            {
                                let a = &mut s.players.get_mut(&attacker).unwrap().items;
                                a.push(item);
                                if let Some(i) = attacker_del_idx {
                                    a.remove(i);
                                }
                            }
                            {
                                let d = &mut s.players.get_mut(&defender).unwrap().items;
                                d.remove(defender_del_idx);
                                if let Some(i) = give_back {
                                    d.push(i);
                                }
                            }
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                    TurnState::WaitingForQuickblink(s.next_player(attacker))
                }
            }
            _ => unimplemented!(),
        };
        Ok(())
    }

    fn new(mut players: Vec<Player>, rng: &mut impl Rng) -> State {
        assert!(players.len() >= 3); // TODO: dreier spiel in sinnvoll
        // TODO: das vmtl falsch
        let mut start_items = [
            Item::Key,
            Item::Key,
            Item::Key,
            Item::Goblet,
            Item::Goblet,
            Item::Goblet,
            Item::BlackPearl, 
            Item::Dagger,
            Item::Gloves,
            Item::PoisonRing,
            Item::CastingKnives,
            Item::Whip,
        ];
        let mut other_items = vec![
            Item::Priviledge,
            Item::Monocle,
            Item::BrokenMirror,
            Item::Sextant,
            Item::Coat,
            Item::Tome,
            Item::CoatOfArmorOfTheLoge        
        ];
        let mut jobs = vec![
            Job::Thug,
            Job::GrandMaster,
            Job::Bodyguard,
            Job::Duelist,
            Job::PoisonMixer,
            Job::Doctor,
            Job::Priest,
            Job::Hypnotist,
            Job::Diplomat,
            Job::Clairvoyant,        
        ];
        let instances_per_faction = (players.len() + 1) / 2;
        let mut factions: Vec<_> = iter::repeat(Faction::Order).take(instances_per_faction)
            .chain(iter::repeat(Faction::Brotherhood).take(instances_per_faction)).collect();
        let (start_items, other_start_items) = start_items.partial_shuffle(rng, players.len() - 2);
        let mut actual_start_items = vec![Item::BagGoblet, Item::BagKey];
        actual_start_items.extend(start_items.iter().copied());
        actual_start_items.shuffle(rng);
        players.shuffle(rng);
        assert_eq!(players.len(), actual_start_items.len());

        other_items.extend(other_start_items.iter().copied());
        other_items.shuffle(rng);

        let (factions, _) = factions.partial_shuffle(rng, players.len());

        let (player_jobs, job_stack) = jobs.partial_shuffle(rng, players.len());

        State {
            game: GameState {
                item_stack: other_items,
                job_stack: job_stack.iter().copied().collect(),
                players: players.iter().zip(actual_start_items).zip(player_jobs).zip(factions)
                    .map(|(((&player, item), &mut job), &mut faction)| (player, PlayerState { faction, job, job_is_visible: false, items: vec![item] })).collect(),
            },
            turn: TurnState::WaitingForQuickblink(players[0]),
        }
    }
}

fn main() {
    /*
    println!("{}", serde_json::to_string(&Command::Pass).unwrap());
    println!("{}", serde_json::to_string(&Command::AnnounceVictory { teammates: vec![Player::Gundla, Player::Sarah] }).unwrap());
    println!("{}", serde_json::to_string(&Command::DeclareSupport { support: AttackSupport::Attack }).unwrap());
    println!("{}", serde_json::to_string(&Command::ItemOrJob { buff: None }).unwrap());
    println!("{}", serde_json::to_string(&Command::ItemOrJob { buff: Some(BuffSource::Item(Item::Gloves)) }).unwrap());
    println!("{}", serde_json::to_string(&Command::ItemOrJob { buff: Some(BuffSource::Job(Job::Bodyguard)) }).unwrap());
    */
    let mut rng = StdRng::seed_from_u64(42);
    let mut state = State::new(vec![Player::Gundla, Player::Marie, Player::Zacharias, Player::Sarah].into_iter().collect(), &mut rng);
    println!("{:#?}", state);

    state.apply_command(Player::Marie, Command::Pass).unwrap();

    state.apply_command(Player::Zacharias, Command::InitiateAttack { player: Player::Marie }).unwrap();
    state.apply_command(Player::Marie, Command::UsePriest { priest: false }).unwrap();
    state.apply_command(Player::Gundla, Command::DeclareSupport { support: AttackSupport::Abstain }).unwrap();
    println!("{:#?}", state);
    state.apply_command(Player::Sarah, Command::DeclareSupport { support: AttackSupport::Attack }).unwrap();
    state.apply_command(Player::Zacharias, Command::Hypnotize { target: None }).unwrap();
    state.apply_command(Player::Marie, Command::ItemOrJob { buff: None }).unwrap();
    state.apply_command(Player::Gundla, Command::ItemOrJob { buff: None }).unwrap();
    state.apply_command(Player::Zacharias, Command::ItemOrJob { buff: Some(BuffSource::Item(Item::PoisonRing)) }).unwrap();
    state.apply_command(Player::Sarah, Command::ItemOrJob { buff: None }).unwrap();
    state.apply_command(Player::Zacharias, Command::ItemOrJob { buff: None }).unwrap();
    state.apply_command(Player::Marie, Command::ItemOrJob { buff: None }).unwrap();
    state.apply_command(Player::Gundla, Command::ItemOrJob { buff: None }).unwrap();
    
    state.apply_command(Player::Zacharias, Command::ClaimReward { steal_items: false }).unwrap();
    state.apply_command(Player::Zacharias, Command::DoneLookingAtThings).unwrap();

    state.apply_command(Player::Gundla, Command::OfferTrade { target: Player::Sarah, item: Item::BagKey }).unwrap();
    state.apply_command(Player::Sarah, Command::RejectTrade).unwrap();

    state.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Zacharias, item: Item::BagGoblet }).unwrap();
    state.apply_command(Player::Zacharias, Command::AcceptTrade { item: Item::PoisonRing }).unwrap();

    state.apply_command(Player::Marie, Command::AnnounceVictory { teammates: vec![] }).unwrap();

    assert_eq!(state.turn, TurnState::GameOver { winner: Faction::Brotherhood });
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn teststate() -> State {
        unimplemented!();
    }
    #[test]
    fn test_add() {
        let mut s = teststate();
        s.apply_command(Player::Gundla, Command::Pass).unwrap();
        s.apply_command(Player::Gundla, Command::Pass).unwrap();
    }
}