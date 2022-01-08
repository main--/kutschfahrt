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

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
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
    #[error("The game is over")]
    GameOver,
    #[error("The job {0:?} does not exist in the job stack")]
    InvalidJobError(Job),
    #[error("You have the black pearl and may not announce victory")]
    BlackPearl,
    #[error("You have already used this buff")]
    DuplicateBuffUsage,
}

impl From<JobUseError> for CommandError {
    fn from(_: JobUseError) -> CommandError {
        CommandError::JobError
    }
}

fn inventory_limit(players: usize) -> usize {
    match players {
        i if i < 3 => panic!("invalid player count"),
        3 => 8,
        4 => 6,
        _ => 5,
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
            TurnState::GameOver { .. } => return Err(CommandError::GameOver),
            TurnState::WaitingForQuickblink(p) => {
                if actor != p {
                    return Err(CommandError::NotYourTurn);
                }

                match c {
                    Command::Pass => {
                        TurnState::WaitingForQuickblink(s.p.next_player(p))
                    }
                    Command::AnnounceVictory { mut teammates } => {
                        let actor_player = s.p.player(actor);
                        if actor_player.items.contains(&Item::BlackPearl) {
                            return Err(CommandError::BlackPearl);
                        }
                        let faction = actor_player.faction;
                        let required_items = match faction {
                            Faction::Order => [Item::Key, Item::BagKey],
                            Faction::Brotherhood => [Item::Goblet, Item::BagGoblet],
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
                        if actor == target {
                            return Err(CommandError::InvalidTargetPlayer);
                        }
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

                            TurnState::Attacking { attacker, defender, state: AttackState::PayingPriest { priest: actor } }
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
                AttackState::PayingPriest { priest } => {
                    match c {
                        Command::PayPriest { item } if actor == attacker => {
                            let mut attacker_state = s.p.player_mut(attacker);
                            let mut priest_state = s.p.player_mut(priest);
                            let index = attacker_state.items.iter().position(|&i| i == item).ok_or(CommandError::InvalidItemError(item))?;
                            attacker_state.items.remove(index);
                            priest_state.items.push(item);
                            let next_player = s.p.next_player(attacker);
                            if priest_state.items.len() > inventory_limit(s.p.players.len()) {
                                TurnState::DonatingItem { donor: priest, followup: ItemDonationFollowup::NextPlayer(next_player) }
                            } else {
                                TurnState::WaitingForQuickblink(next_player)
                            }
                        }
                        Command::PayPriest { .. } => return Err(CommandError::NotYourTurn),
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
                                s.p.player_mut(actor).use_job(Job::Hypnotist)?;
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

                    match c {
                        // TODO: We might wanna warn the player if he specifies a target for a buff that doesn't need a target
                        Command::ItemOrJob { buff: None, target: _ } => {
                            passed.insert(actor);
                            let required_passes = s.p.players.len();
                            if passed.len() == required_passes {
                                let score: BuffScore = buffs.iter().map(|b| b.raw_score)
                                    .chain(votes.values().map(|v| v.vote_value())).sum();

                                if score == 0 {
                                    let next_player = s.p.next_player(attacker);
                                    if let Some(drawn_item) = s.item_stack.pop() {
                                        let mut player_state = s.p.player_mut(attacker);
                                        player_state.items.push(drawn_item);
                                        if player_state.items.len() > inventory_limit(s.p.players.len()) {
                                            TurnState::DonatingItem { donor: attacker, followup: ItemDonationFollowup::NextPlayer(next_player) }
                                        } else {
                                            TurnState::WaitingForQuickblink(next_player)
                                        }
                                    } else {
                                        TurnState::WaitingForQuickblink(next_player)
                                    }
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
                                BuffSource::Job(x) => s.p.player_mut(actor).use_job(x)?,
                                BuffSource::Item(x) => {
                                    if !s.p.player(actor).items.contains(&x) {
                                        return Err(CommandError::InvalidItemError(x))
                                    }
                                }
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

                                    // check if duplicate buff is being used
                                    if buffs.iter().any(|b| b.source == buff) {
                                        return Err(CommandError::DuplicateBuffUsage);
                                    }

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
                    let next_player = s.p.next_player(attacker);
                    match c {
                        Command::DoneLookingAtThings if !steal_items => TurnState::WaitingForQuickblink(next_player),
                        Command::StealItem { item, give_back } if steal_items => {
                            let mut attacker_state = s.p.player_mut(attacker);
                            let mut defender_state = s.p.player_mut(defender);

                            if give_back.is_some() != (defender_state.items.len() == 1) {
                                // violating giveback/donate game rules
                                return Err(CommandError::InvalidStealCommand);
                            }

                            // this whole structure is a bit more complex than it needs to be because we want
                            // to not modify the game state until we have determined that everything is in order

                            let defender_del_idx = defender_state.items.iter().position(|x| *x == item).ok_or(CommandError::InvalidStealCommand)?;
                            let attacker_del_idx = match give_back {
                                None => None,
                                // small complication here because we need to let you donate the item you are stealing
                                Some(i) => Some(attacker_state.items.iter().chain(iter::once(&item)).position(|x| *x == i).ok_or(CommandError::InvalidStealCommand)?),
                            };


                            // only now that everything is verified and valid can we actually modify the game state
                            attacker_state.items.push(item);
                            if let Some(i) = attacker_del_idx {
                                attacker_state.items.remove(i);
                            }

                            defender_state.items.remove(defender_del_idx);
                            if let Some(i) = give_back {
                                defender_state.items.push(i);
                            }

                            if attacker_state.items.len() > inventory_limit(s.p.players.len()) {
                                TurnState::DonatingItem { donor: attacker, followup: ItemDonationFollowup::NextPlayer(next_player) }
                            } else {
                                TurnState::WaitingForQuickblink(next_player)
                            }
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                }
            }
            TurnState::TradePending { offerer, target, item } => {
                let mut newstate = None;
                match c {
                    _ if actor != target => return Err(CommandError::NotYourTurn),
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
                            let np = s.p.players.len();
                            newstate = try_resolve_trade_trigger(item, &mut s.item_stack, &mut offerer_state, &mut target_state, np)
                                    .map(|trigger| match trigger {
                                        Ok(trigger) => TurnState::ResolvingTradeTrigger { offerer, target, next_item: Some(item2), trigger },
                                        Err(NeedDonation) => TurnState::DonatingItem { donor: offerer, followup: ItemDonationFollowup::TradeTriggers { offerer, target, item: item2 } },
                                    })
                                .or_else(|| try_resolve_trade_trigger(item2, &mut s.item_stack, &mut target_state, &mut offerer_state, np)
                                    .map(|trigger| match trigger {
                                        Ok(trigger) => TurnState::ResolvingTradeTrigger { offerer, target, next_item: None, trigger },
                                        Err(NeedDonation) => TurnState::DonatingItem { donor: target, followup: ItemDonationFollowup::NextPlayer(s.p.next_player(offerer)) },
                                    }));
                        }
                    }
                    Command::RejectTrade => (),
                    _ => return Err(CommandError::InvalidCommandInThisContext),
                }
                newstate.unwrap_or(TurnState::WaitingForQuickblink(s.p.next_player(offerer)))
            }
            TurnState::ResolvingTradeTrigger { offerer, target, next_item, trigger } => {
                let responsible_player = if next_item.is_some() { offerer } else { target };
                match trigger {
                    TradeTriggerState::Priviledge | TradeTriggerState::Monocle => {
                        if actor != responsible_player { return Err(CommandError::NotYourTurn); }

                        match c {
                            Command::DoneLookingAtThings => None,
                            _ => return Err(CommandError::InvalidCommandInThisContext),
                        }
                    }
                    TradeTriggerState::Coat => {
                        if actor != responsible_player { return Err(CommandError::NotYourTurn); }

                        match c {
                            Command::PickNewJob { job } => {
                                match s.job_stack.iter().position(|&j| j == job) {
                                    Some(i) => {
                                        let mut p = s.p.player_mut(actor);
                                        std::mem::swap(&mut s.job_stack[i], &mut p.job);
                                        None
                                    }
                                    None => return Err(CommandError::InvalidJobError(job)),
                                }
                            }
                            _ => return Err(CommandError::InvalidCommandInThisContext),
                        }
                    }
                    TradeTriggerState::Sextant { is_forward: None, .. } if actor != responsible_player => return Err(CommandError::NotYourTurn),
                    TradeTriggerState::Sextant { item_selections, is_forward: None } => {
                        match c {
                            Command::SetSextantDirection { forward } => Some(TurnState::ResolvingTradeTrigger { offerer, target, next_item, trigger: TradeTriggerState::Sextant { item_selections, is_forward: Some(forward) } }),
                            _ => return Err(CommandError::InvalidCommandInThisContext),
                        }
                    }
                    TradeTriggerState::Sextant { mut item_selections, is_forward: Some(forward) } => {
                        match c {
                            Command::SelectSextantItem { item } if !item_selections.contains_key(&actor) => {
                                if !s.p.player(actor).items.contains(&item) {
                                    return Err(CommandError::InvalidItemError(item));
                                }
                                item_selections.insert(actor, item);

                                if item_selections.len() == s.p.players.len() {
                                    // everybody has made their choice!
                                    fn eval_sextant<'a>(sels: &HashMap<Player, Item>, i: impl Iterator<Item=(&'a Player, &'a RefCell<PlayerState>)>) {
                                        let mut i = i.peekable();
                                        while let Some(((&px, sx), &(_, sy))) = i.next().and_then(|x| i.peek().map(|y| (x, y))) {
                                            // move px's selection from px's inventory to py's inventory
                                            let item = sels.get(&px).unwrap();
                                            let mut xstate = sx.borrow_mut();
                                            let xindex = xstate.items.iter().position(|i| i == item).unwrap();
                                            xstate.items.remove(xindex);

                                            sy.borrow_mut().items.push(*item);
                                        }
                                    }
                                    let players_iter = s.p.players.iter();
                                    let looped_iter = players_iter.clone().chain(players_iter.take(1));
                                    if forward {
                                        eval_sextant(&item_selections, looped_iter);
                                    } else {
                                        eval_sextant(&item_selections, looped_iter.rev());
                                    }

                                    None
                                } else {
                                    Some(TurnState::ResolvingTradeTrigger { offerer, target, next_item, trigger: TradeTriggerState::Sextant { item_selections, is_forward: Some(forward) } })
                                }
                            }
                            _ => return Err(CommandError::InvalidCommandInThisContext),
                        }
                    }
                }.or_else(|| {
                    next_item.and_then(|ni| {
                        let (mut offerer_state, mut target_state) = s.p.player_pair_mut(offerer, target);
                        try_resolve_trade_trigger(ni, &mut s.item_stack, &mut target_state, &mut offerer_state, s.p.players.len())
                    })
                        .map(|trigger| match trigger {
                            Ok(trigger) => TurnState::ResolvingTradeTrigger { offerer, target, next_item: None, trigger },
                            Err(NeedDonation) => TurnState::DonatingItem { donor: target, followup: ItemDonationFollowup::NextPlayer(s.p.next_player(offerer)) },
                        })
                }).unwrap_or(TurnState::WaitingForQuickblink(s.p.next_player(offerer)))
            }
            TurnState::DonatingItem { donor, followup } => {
                if actor != donor {
                    return Err(CommandError::NotYourTurn);
                }
                match c {
                    Command::DonateItem { target, item } => {
                        {
                            let mut donor_state = s.p.player_mut(donor);
                            let mut target_state = s.p.player_mut(target);
                            let index = donor_state.items.iter().position(|&i| i == item).ok_or(CommandError::InvalidItemError(item))?;
                            donor_state.items.remove(index);
                            target_state.items.push(item);
                        }

                        match followup {
                            ItemDonationFollowup::NextPlayer(p) => TurnState::WaitingForQuickblink(p),
                            ItemDonationFollowup::TradeTriggers { offerer, target, item } => {
                                // at this point we expect that offerer and target are still original (and hence we are dealing with the item that the target is passing to the offerer)
                                dbg!(offerer, target, item);
                                let mut ostate = s.p.player_mut(offerer);
                                let mut tstate = s.p.player_mut(target);
                                try_resolve_trade_trigger(item, &mut s.item_stack, &mut tstate, &mut ostate, s.p.players.len())
                                    .map(|trigger| match trigger {
                                        Ok(trigger) => TurnState::ResolvingTradeTrigger { offerer, target, next_item: None, trigger },
                                        Err(NeedDonation) => TurnState::DonatingItem { donor: target, followup: ItemDonationFollowup::NextPlayer(s.p.next_player(offerer)) },
                                    })
                                    .unwrap_or(TurnState::WaitingForQuickblink(s.p.next_player(offerer)))
                            }
                        }
                    }
                    _ => return Err(CommandError::InvalidCommandInThisContext),
                }
            }
        };
        Ok(())
    }

    pub fn new(mut players: Vec<Player>, rng: &mut impl Rng) -> State {
        //players.push(Player::Zacharias);
        //assert!(players.len() >= 3); // TODO: dreier spiel in sinnvoll
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
            &TurnState::ResolvingTradeTrigger { offerer, target, ref trigger, next_item } => {
                let (relevant, other) = if next_item.is_some() { (offerer, target) } else { (target, offerer) };
                let trigger = match trigger {
                    // only the relevant player is allowed to see the respective info
                    TradeTriggerState::Priviledge if relevant == p =>
                        PerspectiveTradeTriggerState::Priviledge { items: Some(self.game.p.player(other).items.clone()) },
                    TradeTriggerState::Priviledge => PerspectiveTradeTriggerState::Priviledge { items: None },
                    TradeTriggerState::Monocle if relevant == p =>
                        PerspectiveTradeTriggerState::Monocle { faction: Some(self.game.p.player(other).faction) },
                    TradeTriggerState::Monocle => PerspectiveTradeTriggerState::Monocle { faction: None },
                    TradeTriggerState::Coat if relevant == p =>
                        PerspectiveTradeTriggerState::Coat { available_jobs: Some(self.game.job_stack.clone()) },
                    TradeTriggerState::Coat => PerspectiveTradeTriggerState::Coat { available_jobs: None },
                    &TradeTriggerState::Sextant { ref item_selections, is_forward } =>
                        // only show the item you selected (so you know that you selected it)
                        PerspectiveTradeTriggerState::Sextant { item_selections: item_selections.iter().filter(|&(&k, _)| k == p).map(|(&k, &v)| (k, v)).collect(), is_forward },
                };
                ResolvingTradeTrigger { offerer, target, trigger, is_first_item: next_item.is_some() }
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
            &TurnState::DonatingItem { donor, .. } => PerspectiveTurnState::DonatingItem { donor },
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

struct NeedDonation;
fn try_resolve_trade_trigger(
    item: Item,
    item_stack: &mut Vec<Item>,
    offerer_state: &mut PlayerState,
    target_state: &mut PlayerState,
    num_players: usize
) -> Option<Result<TradeTriggerState, NeedDonation>> {
    match item {
        Item::BagKey | Item::BagGoblet => {
            if let Some(i) = item_stack.pop() {
                offerer_state.items.push(i);
                if offerer_state.items.len() > inventory_limit(num_players) {
                    return Some(Err(NeedDonation));
                }
            }
            None
        }
        Item::Priviledge => Some(Ok(TradeTriggerState::Priviledge)),
        Item::Monocle => Some(Ok(TradeTriggerState::Monocle)),
        Item::Sextant => Some(Ok(TradeTriggerState::Sextant { item_selections: HashMap::new(), is_forward: None })),
        Item::Coat => Some(Ok(TradeTriggerState::Coat)),
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
mod tests;

// https://silo.tips/download/die-kutschfahrt-zur-teufelsburg-autoren-michael-palm-und-lukas-zach