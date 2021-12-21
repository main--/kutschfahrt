#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::iter;
use std::ops::{Deref, DerefMut};

use indexmap::IndexMap;
use serde::{Serialize, Deserialize};
use rand::prelude::*;

use web_protocol::*;


#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    p: GameStatePlayers,
    item_stack: Vec<Item>,
    job_stack: Vec<Job>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GameStatePlayers {
    #[serde(with = "indexmap::serde_seq")]
    players: IndexMap<Player, RefCell<PlayerState>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    game: GameState,
    pub turn: TurnState,
}

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("Not your turn")]
    NotYourTurn,
    #[error("Invalid command in this context")]
    InvalidCommandInThisContext,
    #[error("Invalid target player")]
    InvalidTargetPlayer,
    #[error("You have already passed")]
    YouHaveAlreadyPassed,
    #[error("You have no part in this struggle")]
    YouAbstained,
    #[error("Invalid steal command")]
    InvalidStealCommand,
    #[error("Not your job or job already used")]
    JobError,
    #[error("This item {0:?} is not a valid choice")]
    InvalidItemError(Item),
}

impl From<JobUseError> for CommandError {
    fn from(_: JobUseError) -> CommandError {
        CommandError::JobError
    }
}

impl GameStatePlayers {
    fn next_player(&self, p: Player) -> Player {
        let index = self.players.get_index_of(&p).expect("Invalid player");
        let next_index = (index + 1) % self.players.len();
        *self.players.get_index(next_index).unwrap().0
    }
    fn player<'a>(&'a self, p: Player) -> impl Deref<Target=PlayerState> + 'a {
        self.players.get(&p).unwrap().borrow()
    }
    fn player_mut<'a>(&'a self, p: Player) -> impl DerefMut<Target=PlayerState> + 'a {
        self.players.get(&p).unwrap().borrow_mut()
    }
    fn player_pair_mut<'a>(&'a self, a: Player, b: Player) -> (impl DerefMut<Target=PlayerState> + 'a, impl DerefMut<Target=PlayerState> + 'a) {
        (self.players.get(&a).unwrap().borrow_mut(), self.players.get(&b).unwrap().borrow_mut())
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
    pub fn apply_command(&mut self, actor: Player, c: Command) -> Result<(), CommandError> {
        let s = &mut self.game;
        self.turn = match self.turn.clone() {
            TurnState::WaitingForQuickblink(p) => {
                if actor != p {
                    return Err(CommandError::NotYourTurn);
                }

                match c {
                    Command::Pass => {
                        TurnState::WaitingForQuickblink(s.p.next_player(p))
                    }
                    Command::AnnounceVictory { mut teammates } => {
                        let faction = s.p.player(actor).faction;
                        let required_items = match faction {
                            Faction::Order => [Item::Goblet, Item::BagGoblet],
                            Faction::Brotherhood => [Item::Key, Item::BagKey],
                        };

                        let required_items = if s.item_stack.is_empty() {
                            &required_items[..]
                        } else {
                            &required_items[..1]
                        };

                        teammates.push(actor);
                        let mut victory = true;
                        let mut total_victory_items = 0;
                        for t in teammates {
                            let ts = s.p.player(t);
                            let victory_items = ts.items.iter().copied().filter(|i| required_items.contains(i)).count();
                            if victory_items == 0 || ts.faction != faction {
                                victory = false;
                                break;
                            }
                            total_victory_items += victory_items;
                        }
                        victory &= total_victory_items >= 3;

                        if victory {
                            TurnState::GameOver { winner: faction }
                        } else {
                            let winner = match faction {
                                Faction::Order => Faction::Brotherhood,
                                Faction::Brotherhood => Faction::Order,
                            };
                            TurnState::GameOver { winner }
                        }
                    }
                    Command::OfferTrade { target, item } => {
                        if s.p.player(actor).items.iter().all(|&i| i != item) {
                            return Err(CommandError::InvalidItemError(item));
                        }
                        TurnState::TradePending { offerer: actor, target, item }
                    }
                    Command::InitiateAttack { player } => {
                        if !s.p.players.contains_key(&player) || actor == player {
                            return Err(CommandError::InvalidTargetPlayer);
                        }
                        TurnState::Attacking {
                            attacker: actor,
                            defender: player,
                            state: AttackState::WaitingForPriest { passed: HashSet::new() },
                        }
                    }
                    _ => return Err(CommandError::InvalidCommandInThisContext),
                }
            }
            TurnState::Attacking { attacker, defender, state } => match state {
                AttackState::WaitingForPriest { mut passed } => {
                    match c {
                        Command::UsePriest { priest: true } => {
                            if passed.contains(&actor) {
                                return Err(CommandError::YouHaveAlreadyPassed);
                            }

                            let mut defp = s.p.player_mut(actor);
                            defp.use_job(Job::Priest)?;

                            // TODO: resolve priest usage (needs new state I think)
                            unimplemented!();
                        },
                        Command::UsePriest { priest: false } => {
                            passed.insert(actor);
                            let state = if passed.len() == s.p.players.len() {
                                AttackState::DeclaringSupport(HashMap::new())
                            } else {
                                AttackState::WaitingForPriest { passed }
                            };
                            TurnState::Attacking { attacker, defender, state }
                        },
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                }
                AttackState::DeclaringSupport(mut votes) => {
                    let next_voter = s.p.attack_supporters(attacker, defender).nth(votes.len()).unwrap();
                    if actor != next_voter {
                        return Err(CommandError::NotYourTurn);
                    }
                    match c {
                        Command::DeclareSupport { support } => {
                            votes.insert(actor, support);
                            if votes.len() == s.p.players.len() - 2 {
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
                AttackState::ItemsOrJobs { mut votes, mut passed, mut buffs } => {
                    if passed.contains(&actor) {
                        return Err(CommandError::YouHaveAlreadyPassed);
                    }
                    if votes.get(&actor) == Some(&AttackSupport::Abstain) {
                        return Err(CommandError::YouAbstained);
                    }

                    match c {
                        // TODO: We might wanna warn the player if he specifies a target for a buff that doesn't need a target
                        Command::ItemOrJob { buff: None, target: _ } => {
                            passed.insert(actor);
                            let required_passes = votes.values().filter(|&n| *n != AttackSupport::Abstain).count() + 2;
                            if passed.len() == required_passes {
                                let score: BuffScore = buffs.iter().map(|b| b.raw_score)
                                    .chain(votes.values().map(|v| v.vote_value())).sum();

                                if score == 0 {
                                    if let Some(drawn_item) = s.item_stack.pop() {
                                        // TODO: handle item limit
                                        s.p.player_mut(attacker).items.push(drawn_item)
                                    }
                                    TurnState::WaitingForQuickblink(s.p.next_player(attacker))
                                } else {
                                    let winner = if score > 0 {
                                        AttackWinner::Attacker
                                    } else {
                                        AttackWinner::Defender
                                    };
                                    TurnState::Attacking { attacker, defender, state: AttackState::Resolving { winner } }
                                }
                            } else {
                                TurnState::Attacking { attacker, defender, state: AttackState::ItemsOrJobs { votes, passed, buffs } }
                            }
                        }
                        Command::ItemOrJob { buff: Some(buff), target } => {
                            // Detemine actor's role in the struggle
                            let role = if actor == attacker {
                                AttackRole::Attacker
                            } else if actor == defender {
                                AttackRole::Defender
                            } else {
                                AttackRole::AttackSupport(votes[&actor])
                            };

                            // Check if using this buff is vaild for the player
                            match buff {
                                BuffSource::Item(x) if s.p.player(actor).items.contains(&x) => (),
                                BuffSource::Job(x) => s.p.player_mut(actor).use_job(x)?,
                                BuffSource::Item(x) => return Err(CommandError::InvalidItemError(x))
                            }

                            // resolve triggers
                            match buff {
                                // triggers that end the fight:
                                BuffSource::Job(Job::Doctor) => {
                                    TurnState::WaitingForQuickblink(s.p.next_player(attacker))
                                }
                                BuffSource::Job(Job::PoisonMixer) => {
                                    let winner = match target {
                                        Some(x) if x == attacker => AttackWinner::Attacker,
                                        Some(x) if x == defender => AttackWinner::Defender,
                                        _ => return Err(CommandError::InvalidCommandInThisContext)
                                    };
                                    TurnState::Attacking { attacker, defender, state: AttackState::Resolving { winner } }
                                }
                                buff => {
                                    // Check if using this buff is vaild for the player's role
                                    let raw_score = buff.raw_score(role).ok_or(CommandError::InvalidCommandInThisContext)?;

                                    // triggers that don't end the fight:
                                    if let BuffSource::Job(Job::Duelist) = &buff {
                                        for vote in votes.values_mut() {
                                            *vote = AttackSupport::Abstain;
                                        }
                                        buffs.retain(|x| x.user == attacker || x.user == defender);
                                    }

                                    buffs.push(Buff { user: actor, raw_score, source: buff });
                                    passed.clear();

                                    TurnState::Attacking { attacker, defender, state: AttackState::ItemsOrJobs { votes, passed, buffs } }
                                }
                            }
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }

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
                            // TODO: implement inventory limit and item donation

                            let (mut attacker_state, mut defender_state) = s.p.player_pair_mut(attacker, defender);
                            if give_back.is_some() != (defender_state.items.len() == 1) {
                                // give back an item exactly when defender has exactly 1 item
                                return Err(CommandError::InvalidStealCommand);
                            }
                            let defender_del_idx = defender_state.items.iter().position(|x| *x == item).ok_or(CommandError::InvalidStealCommand)?;
                            let attacker_del_idx = match give_back {
                                None => None,
                                Some(i) => Some(attacker_state.items.iter().position(|x| *x == i).ok_or(CommandError::InvalidStealCommand)?),
                            };

                            // only now that everything is verified and valid can we actually modify the game state
                            attacker_state.items.push(item);
                            if let Some(i) = attacker_del_idx {
                                attacker_state.items.remove(i);
                            }

                            defender_state.items.remove(defender_del_idx);
                            if let Some(i) = give_back {
                                attacker_state.items.push(i);
                            }
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                    TurnState::WaitingForQuickblink(s.p.next_player(attacker))
                }
            }
            TurnState::TradePending { offerer, target, item } => {
                // TODO: trade triggers not yet implemented
                let mut newstate = None;
                match c {
                    Command::AcceptTrade { item: item2 } => {
                        let items = [item, item2];
                        if !s.item_stack.is_empty() && items.contains(&Item::BagGoblet) && items.contains(&Item::BagKey) {
                            // TODO: is item stack even relevant here?
                            return Err(CommandError::InvalidItemError(item2)); // can't swap bag for bag
                        }

                        let (mut offerer_state, mut target_state) = s.p.player_pair_mut(offerer, target);

                        let idx_offerer = offerer_state.items.iter().position(|&i| i == item)
                            .expect("We should not have allowed them to offer an item they don't even have.");
                        let idx_target = match target_state.items.iter().position(|&i| i == item2) {
                            Some(i) => i,
                            None => return Err(CommandError::InvalidItemError(item2)),
                        };

                        // swap the items
                        std::mem::swap(&mut offerer_state.items[idx_offerer], &mut target_state.items[idx_target]);
                        // TODO: does this represent the game rules accurately?

                        // no triggers if broken mirror was swapped
                        if ![item, item2].contains(&Item::BrokenMirror) {
                            // triggers for offered item
                            newstate = try_resolve_trade_trigger(item, &mut s.item_stack, &mut offerer_state, &mut target_state).map(|trigger| TurnState::ResolvingTradeTrigger { offerer, target, next_item: Some(item2), trigger })
                                .or_else(|| try_resolve_trade_trigger(item2, &mut s.item_stack, &mut target_state, &mut offerer_state).map(|trigger| TurnState::ResolvingTradeTrigger { offerer, target, next_item: None, trigger }));
                        }
                    }
                    Command::RejectTrade => (),
                    _ => return Err(CommandError::InvalidCommandInThisContext),
                }
                newstate.unwrap_or(TurnState::WaitingForQuickblink(s.p.next_player(offerer)))
            }
            _ => unimplemented!(),
        };
        Ok(())
    }

    pub fn new(mut players: Vec<Player>, rng: &mut impl Rng) -> State {
        assert!(players.len() >= 3); // TODO: dreier spiel in sinnvoll
        // Das ist jetzt nicht mehr falsch
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
            Item::Priviledge,
            Item::Monocle
        ];
        let mut other_items = vec![
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
                p: GameStatePlayers {
                    players: players.iter().zip(actual_start_items).zip(player_jobs).zip(factions)
                    .map(|(((&player, item), &mut job), &mut faction)| (player, RefCell::new(PlayerState { faction, job, job_is_visible: false, items: vec![item] }))).collect()
                }
            },
            turn: TurnState::WaitingForQuickblink(players[0]),
        }
    }
    pub fn perspective(&self, p: Player) -> Perspective {
        use PerspectiveTurnState::*;
        let turn = match &self.turn {
            &TurnState::WaitingForQuickblink(player) => TurnStart { player },
            &TurnState::GameOver { winner } => GameOver { winner },
            &TurnState::TradePending { offerer, target, item } if target == p => TradePending { offerer, target, item: Some(item) },
            &TurnState::TradePending { offerer, target, .. } => TradePending { offerer, target, item: None },
            &TurnState::ResolvingTradeTrigger { offerer, target, ref trigger, .. } => {
                let trigger = match trigger {
                    TradeTriggerState::Sextant { .. } =>
                        TradeTriggerState::Sextant { item_selections: HashMap::new() },
                    t => t.clone(),
                };
                ResolvingTradeTrigger { offerer, target, trigger }
            }
            &TurnState::Attacking { attacker, defender, ref state } => {
                let myself = if p == attacker {
                    Some(AttackWinner::Attacker)
                } else if p == defender {
                    Some(AttackWinner::Defender)
                } else {
                    None
                };

                let state = match (state, myself) {
                    // if we're in FinishResolving AND I am the winner, then I get to see extra info
                    (&AttackState::FinishResolving { winner, steal_items }, Some(me)) if me == winner => {
                        let victim = match winner {
                            AttackWinner::Attacker => defender,
                            AttackWinner::Defender => attacker,
                        };
                        let victim = self.game.p.player(victim);
                        if steal_items {
                            PerspectiveAttackState::FinishResolvingItems { target_items: victim.items.clone() }
                        } else {
                            PerspectiveAttackState::FinishResolvingCredentials { target_faction: victim.faction, target_job: victim.job }
                        }
                    }
                    _ => PerspectiveAttackState::Normal(state.clone()),
                };

                Attacking { attacker, defender, state }
            }
        };
        Perspective {
            you: self.game.p.player(p).clone(),
            your_player_index: self.game.p.players.get_index_of(&p).unwrap(),
            players: self.game.p.players.iter().map(|(&k, v)| {
                let v = v.borrow();
                PerspectivePlayer {
                    player: k,
                    job: if v.job_is_visible { Some(v.job) } else { None },
                    item_count: v.items.len(),
                }
            }).collect(),
            item_stack: self.game.item_stack.len(),
            turn,
        }
    }
}

fn try_resolve_trade_trigger(
    item: Item,
    item_stack: &mut Vec<Item>,
    offerer_state: &mut PlayerState,
    target_state: &mut impl DerefMut<Target = PlayerState>
) -> Option<TradeTriggerState> {
    match item {
        Item::BagKey | Item::BagGoblet => {
            if let Some(i) = item_stack.pop() {
                offerer_state.items.push(i);
            }
            None
        }
        Item::Priviledge => Some(TradeTriggerState::Priviledge),
        Item::Monocle => Some(TradeTriggerState::Monocle),
        Item::Sextant => Some(TradeTriggerState::Sextant { item_selections: HashMap::new() }),
        Item::Coat => Some(TradeTriggerState::Coat),
        Item::Tome => {
            std::mem::swap(&mut offerer_state.job, &mut target_state.job);
            offerer_state.job_is_visible = false;
            target_state.job_is_visible = false;
            None
        }
        _ => None,
    }
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
