#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::{iter, cmp};
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
    action_log: Vec<ActionLogEntry>,
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
    #[error("The item forces you to accept this trade")]
    MustAccept,
    #[error("You can't use the Poison Mixer in a fight where you a the attacker or defender")]
    CantPoisonMixYourself,
    #[error("Clairvoyant needs to select exactly two items (unless the stack is almost empty)")]
    WrongNumberOfClairvoyantItems,
    #[error("Solo victory requires at least 3 (mixed) victory items (and coat of arms of the loge)")]
    InvalidLogeVictory,
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
            TurnState::GameOver { .. } => return Err(CommandError::GameOver),
            TurnState::UnsuccessfulDiplomat { diplomat, .. } => {
                if actor != diplomat {
                    return Err(CommandError::NotYourTurn);
                }
                match c {
                    // diplomat was unsuccessful, so we transition to waiting
                    Command::DoneLookingAtThings => TurnState::WaitingForQuickblink(s.p.next_player(diplomat)),
                    _ => return Err(CommandError::InvalidCommandInThisContext),
                }
            }
            TurnState::WaitingForQuickblink(p) | TurnState::WaitingForEndTurn(p) => {
                if actor != p {
                    return Err(CommandError::NotYourTurn);
                }

                let is_end_phase = matches!(self.turn, TurnState::WaitingForEndTurn(_));

                // clairvoyant and diplomat can be issued either before or after the main action.
                let next_turn_player = if is_end_phase {
                    // when used in the end phase, they pass control to the next player.
                    s.p.next_player(p)
                } else {
                    // when used in main phase (i.e. before the main action), they pass control back to the current player
                    p
                };
                // there is no problem with chaining because the jobs are single-use

                match c {
                    Command::Pass => {
                        if !is_end_phase {
                            s.action_log.push(ActionLogEntry::Pass { actor });
                        }
                        // skip end phase (next_turn_player) for this because it would be redundant
                        TurnState::WaitingForQuickblink(s.p.next_player(p))
                    }
                    Command::UseClairvoyant => {
                        s.p.player_mut(p).use_job(Job::Clairvoyant)?;
                        s.action_log.push(ActionLogEntry::UseClairvoyant { actor });
                        TurnState::DoingClairvoyant { clairvoyant: p, next: next_turn_player }
                    }
                    Command::UseDiplomat { target, item, return_item } => {
                        if !s.p.players.contains_key(&target) || actor == target {
                            return Err(CommandError::InvalidTargetPlayer);
                        }

                        s.p.player_mut(p).use_job(Job::Diplomat)?;
                        let (actor_items, target_items) = s.p.player_pair_mut(actor, target);
                        let _return_item_index = actor_items.items.iter().position(|&x| x == return_item).ok_or(CommandError::InvalidItemError(return_item))?;
                        let stack_empty = s.item_stack.is_empty();
                        let resolved_target_item = target_items.items.iter().copied().find(|&x| x == item || (stack_empty && (item == Item::Goblet && x == Item::BagGoblet) || (item == Item::Key && x == Item::BagKey)));
                        drop((actor_items, target_items));

                        s.action_log.push(ActionLogEntry::UseDiplomat { actor, target, item, success: resolved_target_item.is_some() });

                        match resolved_target_item {
                            Some(target_item) => {
                                let next_state = TurnState::WaitingForQuickblink(next_turn_player);
                                perform_trade(s, target, target_item, actor, return_item, next_state)?
                            }
                            None => {
                                TurnState::UnsuccessfulDiplomat { diplomat: actor, target }
                            }
                        }
                    }
                    _ if is_end_phase => return Err(CommandError::InvalidCommandInThisContext),
                    // commands below this check are not valid in the end phase
                    // intended behavior: 99% of the time you will skip end phase by passing.
                    // but it's needed to give you an opportunity for end-of-turn clairvoyant and diplomat

                    Command::AnnounceVictory { flavor } => {
                        let actor_player = s.p.player(actor);
                        s.action_log.push(ActionLogEntry::AnnounceVictory { actor });
                        if actor_player.items.contains(&Item::BlackPearl) {
                            return Err(CommandError::BlackPearl);
                        }
                        let faction = actor_player.effective_faction();
                        let num_faction_members = s.p.players.values().filter(|x| x.borrow().effective_faction() == faction).count();
                        let is_minority_faction = num_faction_members * 2 < s.p.players.len();
                        let required_items: &[_] = match (&flavor, faction) {
                            (VictoryFlavor::Normal { .. }, Faction::Order) => &[Item::Key, Item::BagKey],
                            (VictoryFlavor::Normal { .. }, Faction::Brotherhood) => &[Item::Goblet, Item::BagGoblet],
                            (VictoryFlavor::Loge, _) => &[Item::Key, Item::BagKey, Item::Goblet, Item::BagGoblet],
                        };
                        let mut required_items = required_items.to_vec();

                        if !s.item_stack.is_empty() {
                            required_items.retain(|&x| x != Item::BagKey && x != Item::BagGoblet);
                        }

                        let (loge, mut teammates) = match flavor {
                            VictoryFlavor::Normal { teammates } => (false, teammates),
                            VictoryFlavor::Loge => (true, Vec::new()),
                        };
                        teammates.push(actor);
                        let mut victory = true;
                        let mut total_victory_items = 0;
                        for t in teammates {
                            let ts = s.p.player(t);
                            let victory_items = ts.items.iter().copied().filter(|i| required_items.contains(i)).count();
                            if victory_items == 0 || ts.effective_faction() != faction {
                                victory = false;
                                break;
                            }
                            total_victory_items += victory_items;
                        }
                        let needed_victory_items = if is_minority_faction { 2 } else { 3 };
                        victory &= total_victory_items >= needed_victory_items;

                        let winner = if loge {
                            if !victory || !s.p.player(actor).items.contains(&Item::CoatOfArmorOfTheLoge) {
                                return Err(CommandError::InvalidLogeVictory);
                            }
                            WinningFaction::Traitor(actor)
                        } else if victory {
                            WinningFaction::Normal(faction)
                        } else {
                            WinningFaction::Normal(match faction {
                                Faction::Order => Faction::Brotherhood,
                                Faction::Brotherhood => Faction::Order,
                            })
                        };
                        TurnState::GameOver { winner }
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

                        s.action_log.push(ActionLogEntry::Attack { attacker: actor, target: player });

                        // if the priest card is already publicly visible, skip the priest phase entirely
                        let priest_is_used = s.p.players.values().any(|p| p.borrow().job_is_visible && p.borrow().job == Job::Priest);
                        // in addition, it is safe to auto-pass everyone who already has an open job
                        let passed: HashSet<_> = s.p.players.iter().filter(|(_, v)| v.borrow().job_is_visible).map(|(&k, _)| k).collect();

                        let state = if priest_is_used || passed.len() == s.p.players.len() {
                            AttackState::DeclaringSupport(HashMap::new())
                        } else {
                            AttackState::WaitingForPriest { passed }
                        };
                        TurnState::Attacking {
                            attacker: actor,
                            defender: player,
                            state,
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
                            if priest_state.items.len() > inventory_limit(s.p.players.len()) {
                                TurnState::DonatingItem { donor: priest, followup: FollowupState::end_phase(attacker) }
                            } else {
                                TurnState::WaitingForEndTurn(attacker)
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
                                    if let Some(drawn_item) = s.item_stack.pop() {
                                        let mut player_state = s.p.player_mut(attacker);
                                        player_state.items.push(drawn_item);
                                        if player_state.items.len() > inventory_limit(s.p.players.len()) {
                                            TurnState::DonatingItem { donor: attacker, followup: FollowupState::end_phase(attacker) }
                                        } else {
                                            TurnState::WaitingForEndTurn(attacker)
                                        }
                                    } else {
                                        TurnState::WaitingForEndTurn(attacker)
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

                            // additional poison mixer validation:
                            if let BuffSource::Job(Job::PoisonMixer) = &buff {
                                if let AttackRole::Attacker | AttackRole::Defender = role {
                                    return Err(CommandError::CantPoisonMixYourself);
                                }
                                match target {
                                    Some(x) if [attacker, defender].contains(&x) => (),
                                    _ => return Err(CommandError::InvalidCommandInThisContext),
                                }
                            }

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
                                    TurnState::WaitingForEndTurn(attacker)
                                }
                                BuffSource::Job(Job::PoisonMixer) => {
                                    let winner = match target {
                                        Some(x) if x == attacker => AttackWinner::Attacker,
                                        Some(x) if x == defender => AttackWinner::Defender,
                                        _ => unreachable!(),
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
                            TurnState::Attacking { attacker, defender, state: AttackState::FinishResolving { winner, steal_items, three_player_faction_index: None } }
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                }
                AttackState::FinishResolving { winner, steal_items, three_player_faction_index } => {
                    let (winner_player, loser_player) = match winner {
                        AttackWinner::Attacker => (attacker, defender),
                        AttackWinner::Defender => (defender, attacker),
                    };
                    if actor != winner_player {
                        return Err(CommandError::NotYourTurn);
                    }

                    match c {
                        Command::DoneLookingAtThings if !steal_items => TurnState::WaitingForEndTurn(attacker),
                        Command::StealItem { item, give_back } if steal_items => {
                            let mut winner_state = s.p.player_mut(winner_player);
                            let mut loser_state = s.p.player_mut(loser_player);

                            if give_back.is_some() != (loser_state.items.len() == 1) {
                                // violating giveback/donate game rules
                                return Err(CommandError::InvalidStealCommand);
                            }

                            // this whole structure is a bit more complex than it needs to be because we want
                            // to not modify the game state until we have determined that everything is in order

                            let defender_del_idx = loser_state.items.iter().position(|x| *x == item).ok_or(CommandError::InvalidStealCommand)?;
                            let attacker_del_idx = match give_back {
                                None => None,
                                // small complication here because we need to let you donate the item you are stealing
                                Some(i) => Some(winner_state.items.iter().chain(iter::once(&item)).position(|x| *x == i).ok_or(CommandError::InvalidStealCommand)?),
                            };


                            // only now that everything is verified and valid can we actually modify the game state
                            winner_state.items.push(item);
                            if let Some(i) = attacker_del_idx {
                                winner_state.items.remove(i);
                            }

                            loser_state.items.remove(defender_del_idx);
                            if let Some(i) = give_back {
                                loser_state.items.push(i);
                            }

                            if winner_state.items.len() > inventory_limit(s.p.players.len()) {
                                TurnState::DonatingItem { donor: winner_player, followup: FollowupState::end_phase(attacker) }
                            } else {
                                TurnState::WaitingForEndTurn(attacker)
                            }
                        }
                        Command::ThreePlayerSelectFactionIndex { index } => {
                            if three_player_faction_index.is_some() || matches!(s.p.player(loser_player).faction, FactionKind::Normal(_)) {
                                return Err(CommandError::InvalidCommandInThisContext);
                            }

                            TurnState::Attacking { attacker, defender, state: AttackState::FinishResolving { winner, steal_items, three_player_faction_index: Some(index) } }
                        }
                        _ => return Err(CommandError::InvalidCommandInThisContext),
                    }
                }
            }
            TurnState::TradePending { offerer, target, item } => {
                let mut newstate = TurnState::WaitingForEndTurn(offerer);
                s.action_log.push(ActionLogEntry::TradeOffer { offerer, target, accepted: matches!(c, Command::AcceptTrade { .. }) });
                match c {
                    _ if actor != target => return Err(CommandError::NotYourTurn),
                    Command::AcceptTrade { item: item2 } => {
                        newstate = perform_trade(s, offerer, item, target, item2, newstate)?;
                    }
                    Command::RejectTrade if [Item::BlackPearl, Item::BrokenMirror].contains(&item) => return Err(CommandError::MustAccept),
                    Command::RejectTrade => (),
                    _ => return Err(CommandError::InvalidCommandInThisContext),
                }
                newstate
            }
            TurnState::ResolvingTradeTrigger { giver, receiver, trigger, next_state } => {
                let new_trigger = match trigger {
                    trigger @ (TradeTriggerState::Priviledge | TradeTriggerState::Monocle { .. }) => {
                        if actor != giver { return Err(CommandError::NotYourTurn); }

                        match c {
                            Command::DoneLookingAtThings => Err(next_state),
                            Command::ThreePlayerSelectFactionIndex { index } if
                                matches!(trigger, TradeTriggerState::Monocle { three_player_faction_index: None })
                                && matches!(s.p.player(receiver).faction, FactionKind::ThreePlayer(_))
                                => Ok(TurnState::ResolvingTradeTrigger { giver, receiver, trigger: TradeTriggerState::Monocle { three_player_faction_index: Some(index) }, next_state }),
                            _ => return Err(CommandError::InvalidCommandInThisContext),
                        }
                    }
                    TradeTriggerState::Coat => {
                        if actor != giver { return Err(CommandError::NotYourTurn); }

                        match c {
                            Command::PickNewJob { job } => {
                                match s.job_stack.iter().position(|&j| j == job) {
                                    Some(i) => {
                                        let mut p = s.p.player_mut(actor);
                                        std::mem::swap(&mut s.job_stack[i], &mut p.job);
                                        Err(next_state)
                                    }
                                    None => return Err(CommandError::InvalidJobError(job)),
                                }
                            }
                            _ => return Err(CommandError::InvalidCommandInThisContext),
                        }
                    }
                    TradeTriggerState::Sextant { is_forward: None, .. } if actor != giver => return Err(CommandError::NotYourTurn),
                    TradeTriggerState::Sextant { item_selections, is_forward: None } => {
                        match c {
                            Command::SetSextantDirection { forward } => Ok(TurnState::ResolvingTradeTrigger { giver, receiver, next_state, trigger: TradeTriggerState::Sextant { item_selections, is_forward: Some(forward) } }),
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

                                    Err(next_state)
                                } else {
                                    Ok(TurnState::ResolvingTradeTrigger { giver, receiver, next_state, trigger: TradeTriggerState::Sextant { item_selections, is_forward: Some(forward) } })
                                }
                            }
                            _ => return Err(CommandError::InvalidCommandInThisContext),
                        }
                    }
                };

                match new_trigger {
                    Ok(x) => x,
                    Err(next_state) => resolve_trade_followup(s, next_state),
                }
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

                        s.action_log.push(ActionLogEntry::DonateItem { giver: donor, receiver: target });

                        resolve_trade_followup(s, followup)
                    }
                    _ => return Err(CommandError::InvalidCommandInThisContext),
                }
            }
            TurnState::DoingClairvoyant { clairvoyant, next } => {
                if actor != clairvoyant {
                    return Err(CommandError::NotYourTurn);
                }
                match c {
                    Command::ClairvoyantSetItems { top_items } => {
                        if top_items.len() != cmp::min(2, s.item_stack.len()) {
                            return Err(CommandError::WrongNumberOfClairvoyantItems);
                        }

                        // remove the selected items from the stack
                        for i in &top_items {
                            let idx = s.item_stack.iter().position(|x| x == i).ok_or(CommandError::InvalidItemError(*i))?;
                            s.item_stack.swap_remove(idx);
                        }

                        // shuffle the remaining items
                        s.item_stack.shuffle(&mut rand::thread_rng());

                        // and push the selected ones back on top
                        for i in top_items.into_iter().rev() {
                            s.item_stack.push(i);
                        }

                        TurnState::WaitingForQuickblink(next)
                    }
                    _ => return Err(CommandError::InvalidCommandInThisContext),
                }
            }
        };

        // skip end phase if the player had their job revealed already
        // (in which case it's public information that the only button they have is End Turn)
        if let &TurnState::WaitingForEndTurn(p) = &self.turn {
            if self.game.p.player(p).job_is_visible {
                self.turn = TurnState::WaitingForQuickblink(self.game.p.next_player(p));
            }
        }

        Ok(())
    }

    pub fn new(mut players: Vec<Player>, rng: &mut impl Rng) -> State {
        assert!(players.len() >= 3);

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

        let mut factions: Vec<_>;
        let factions: Box<dyn Iterator<Item=FactionKind>> = if players.len() == 3 {
            factions = iter::repeat(Faction::Order).take(5)
                .chain(iter::repeat(Faction::Brotherhood).take(5)).collect();
            factions.shuffle(rng);
            Box::new(factions.chunks_exact(3).map(|chunk| FactionKind::ThreePlayer(chunk.try_into().unwrap())))
        } else {
            let instances_per_faction = (players.len() + 1) / 2;
            factions = iter::repeat(Faction::Order).take(instances_per_faction)
                .chain(iter::repeat(Faction::Brotherhood).take(instances_per_faction)).collect();
            let (factions, _) = factions.partial_shuffle(rng, players.len());
            Box::new(factions.iter().copied().map(FactionKind::Normal))
        };

        let (start_items, other_start_items) = start_items.partial_shuffle(rng, players.len() - 2);
        let mut actual_start_items = vec![Item::BagGoblet, Item::BagKey];
        actual_start_items.extend(start_items.iter().copied());
        actual_start_items.shuffle(rng);
        players.shuffle(rng);
        assert_eq!(players.len(), actual_start_items.len());

        other_items.extend(other_start_items.iter().copied());
        other_items.shuffle(rng);


        jobs.shuffle(rng);
        let (player_jobs, job_stack) = jobs.split_at_mut(players.len());

        State {
            game: GameState {
                item_stack: other_items,
                job_stack: job_stack.iter().copied().collect(),
                action_log: Vec::new(),
                p: GameStatePlayers {
                    players: players.iter().zip(actual_start_items).zip(player_jobs).zip(factions)
                    .map(|(((&player, item), &mut job), faction)| (player, RefCell::new(PlayerState { faction, job, job_is_visible: false, items: vec![item] }))).collect()
                }
            },
            turn: TurnState::WaitingForQuickblink(players[0]),
        }
    }
    pub fn perspective(&self, p: Player) -> Perspective {
        use PerspectiveTurnState::*;
        let turn = match &self.turn {
            &TurnState::WaitingForQuickblink(player) => TurnStart { player },
            &TurnState::WaitingForEndTurn(player) => TurnEndPhase { player },
            &TurnState::DoingClairvoyant { clairvoyant: c, .. } if c == p => DoingClairvoyant { player: c, item_stack: Some(self.game.item_stack.clone()) },
            &TurnState::DoingClairvoyant { clairvoyant: c, .. } => DoingClairvoyant { player: c, item_stack: None },
            &TurnState::UnsuccessfulDiplomat { diplomat , target } if diplomat == p => UnsuccessfulDiplomat { diplomat, target, inventory: Some(self.game.p.player(target).items.clone()) },
            &TurnState::UnsuccessfulDiplomat { diplomat , target } => UnsuccessfulDiplomat { diplomat, target, inventory: None },
            &TurnState::GameOver { winner } => GameOver { winner },
            &TurnState::TradePending { offerer, target, item } if target == p => TradePending { offerer, target, item: Some(item) },
            &TurnState::TradePending { offerer, target, .. } => TradePending { offerer, target, item: None },
            &TurnState::ResolvingTradeTrigger { giver, receiver, ref trigger, next_state: _ } => {
                let trigger = match trigger {
                    // only the relevant player is allowed to see the respective info
                    TradeTriggerState::Priviledge if giver == p =>
                        PerspectiveTradeTriggerState::Priviledge { items: Some(self.game.p.player(receiver).items.clone()) },
                    TradeTriggerState::Priviledge => PerspectiveTradeTriggerState::Priviledge { items: None },
                    &TradeTriggerState::Monocle { three_player_faction_index } if giver == p =>
                        PerspectiveTradeTriggerState::Monocle { faction: self.game.p.player(receiver).faction_by_index(three_player_faction_index), three_player_faction_index },
                    &TradeTriggerState::Monocle { three_player_faction_index } => PerspectiveTradeTriggerState::Monocle { faction: None, three_player_faction_index },
                    TradeTriggerState::Coat if giver == p =>
                        PerspectiveTradeTriggerState::Coat { available_jobs: Some(self.game.job_stack.clone()) },
                    TradeTriggerState::Coat => PerspectiveTradeTriggerState::Coat { available_jobs: None },
                    &TradeTriggerState::Sextant { ref item_selections, is_forward } =>
                        // only show the item you selected (so you know that you selected it)
                        PerspectiveTradeTriggerState::Sextant { item_selections: item_selections.iter().filter(|&(&k, _)| k == p).map(|(&k, &v)| (k, v)).collect(), is_forward },
                };
                ResolvingTradeTrigger { giver, receiver, trigger }
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
                    (&AttackState::FinishResolving { winner, steal_items, three_player_faction_index }, Some(me)) if me == winner => {
                        let victim = match winner {
                            AttackWinner::Attacker => defender,
                            AttackWinner::Defender => attacker,
                        };
                        let victim = self.game.p.player(victim);
                        if steal_items {
                            PerspectiveAttackState::FinishResolvingItems { target_items: victim.items.clone() }
                        } else {
                            victim
                                .faction_by_index(three_player_faction_index)
                                .map_or(
                                    PerspectiveAttackState::FinishResolvingNeedFactionIndex,
                                    |target_faction| PerspectiveAttackState::FinishResolvingCredentials { target_faction, target_job: victim.job }
                                )
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
            action_log: self.game.action_log.clone(),
        }
    }
}

struct NeedDonation;
fn try_resolve_trade_trigger(
    item: Item,
    item_stack: &mut Vec<Item>,
    offerer_state: &mut PlayerState,
    target_state: &mut PlayerState,
    num_players: usize,
) -> (Option<Result<TradeTriggerState, NeedDonation>>, bool) {
    let mut public_information = true;
    let ret = match item {
        Item::BagKey | Item::BagGoblet => {
            if let Some(i) = item_stack.pop() {
                offerer_state.items.push(i);
                if offerer_state.items.len() > inventory_limit(num_players) {
                    return (Some(Err(NeedDonation)), true);
                }
            } else {
                // if stack is empty, trading bags is hidden information
                public_information = false;
            }
            None
        }
        Item::Priviledge => Some(Ok(TradeTriggerState::Priviledge)),
        Item::Monocle => Some(Ok(TradeTriggerState::Monocle { three_player_faction_index: None })),
        Item::Sextant => Some(Ok(TradeTriggerState::Sextant { item_selections: HashMap::new(), is_forward: None })),
        Item::Coat => Some(Ok(TradeTriggerState::Coat)),
        Item::Tome => {
            std::mem::swap(&mut offerer_state.job, &mut target_state.job);
            offerer_state.job_is_visible = false;
            target_state.job_is_visible = false;
            None
        }
        _ => {
            // trading items that don't have triggers is also hidden info
            public_information = false;
            None
        }
    };
    (ret, public_information)
}
fn resolve_trade_followup(
    s: &mut GameState,
    followup: FollowupState,
) -> TurnState {
    match followup {
        FollowupState::State(s) => *s,
        FollowupState::TradeTriggers { giver, receiver, item, next_state } => {
            let (mut offerer_state, mut target_state) = s.p.player_pair_mut(giver, receiver);
            let (trigger, public) = try_resolve_trade_trigger(item, &mut s.item_stack, &mut *offerer_state, &mut *target_state, s.p.players.len());
            if public {
                // render both types of bags as BagGoblet to ensure we don't leak which one it is
                let item = if item == Item::BagKey { Item::BagGoblet } else { item };
                s.action_log.push(ActionLogEntry::TradeTrigger { giver, receiver, item });
            }
            match trigger {
                None => *next_state,
                Some(Ok(trigger)) => TurnState::ResolvingTradeTrigger { giver, receiver, next_state: FollowupState::State(next_state), trigger },
                Some(Err(NeedDonation)) => TurnState::DonatingItem { donor: receiver, followup: FollowupState::State(next_state) },
            }
        }
    }
}
fn perform_trade(
    s: &mut GameState,
    offerer: Player,
    item: Item,
    target: Player,
    item2: Item,
    next_state: TurnState,
) -> Result<TurnState, CommandError> {
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

    std::mem::swap(&mut offerer_state.items[idx_offerer], &mut target_state.items[idx_target]);
    // TODO: does this represent the game rules accurately?

    Ok(if items.contains(&Item::BrokenMirror) {
        // no triggers if broken mirror was swapped
        next_state
    } else {
        // triggers for offered item
        let np = s.p.players.len();

        let next_state = Box::new(next_state);

        let (trigger, public) = try_resolve_trade_trigger(item, &mut s.item_stack, &mut offerer_state, &mut target_state, np);
        if public {
            s.action_log.push(ActionLogEntry::TradeTrigger { giver: offerer, receiver: target, item });
        }
        match trigger {
            Some(trigger) => {
                let fus = FollowupState::TradeTriggers { giver: target, receiver: offerer, item: item2, next_state };
                match trigger {
                    Ok(trigger) => TurnState::ResolvingTradeTrigger { giver: offerer, receiver: target, next_state: fus, trigger },
                    Err(NeedDonation) => TurnState::DonatingItem { donor: offerer, followup: fus },
                }
            }
            None => {
                let (trigger, public) = try_resolve_trade_trigger(item2, &mut s.item_stack, &mut target_state, &mut offerer_state, np);
                if public {
                    s.action_log.push(ActionLogEntry::TradeTrigger { giver: target, receiver: offerer, item: item2 });
                }
                match trigger {
                    Some(trigger) => {
                        let next_state = FollowupState::State(next_state);
                        match trigger {
                            Ok(trigger) => TurnState::ResolvingTradeTrigger { giver: target, receiver: offerer, trigger, next_state },
                            Err(NeedDonation) => TurnState::DonatingItem { donor: target, followup: next_state },
                        }
                    }
                    None => *next_state,
                }
            }
        }
    })
}



#[cfg(test)]
mod tests;

// https://silo.tips/download/die-kutschfahrt-zur-teufelsburg-autoren-michael-palm-und-lukas-zach