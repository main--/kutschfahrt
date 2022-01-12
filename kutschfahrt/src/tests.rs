use super::*;

fn teststate() -> State {
    State {
        game: GameState {
            p: GameStatePlayers { players: [
                (Player::Sarah, RefCell::new(PlayerState { faction: Faction::Order, job: Job::Duelist, job_is_visible: false, items: vec![Item::BagKey] })),
                (Player::Gundla, RefCell::new(PlayerState { faction: Faction::Brotherhood, job: Job::Clairvoyant, job_is_visible: false, items: vec![Item::BagGoblet] })),
                (Player::Marie, RefCell::new(PlayerState { faction: Faction::Order, job: Job::Thug, job_is_visible: false, items: vec![Item::PoisonRing] })),
                (Player::Zacharias, RefCell::new(PlayerState { faction: Faction::Brotherhood, job: Job::Hypnotist, job_is_visible: false, items: vec![Item::Gloves] })),
            ].into_iter().collect() },
            item_stack: vec![Item::BlackPearl, Item::Dagger],
            job_stack: vec![Job::Doctor],
        },
        turn: TurnState::WaitingForQuickblink(Player::Sarah),
    }
}

#[test]
fn pass() {
    let mut s = teststate();
    s.apply_command(Player::Sarah, Command::Pass).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    s.apply_command(Player::Gundla, Command::Pass).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Marie));
    s.apply_command(Player::Marie, Command::Pass).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Zacharias));
    s.apply_command(Player::Zacharias, Command::Pass).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Sarah));
    s.apply_command(Player::Sarah, Command::Pass).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
}

#[test]
fn wrong_player() {
    let mut s = teststate();
    assert_eq!(s.apply_command(Player::Gundla, Command::Pass), Err(CommandError::NotYourTurn));
}

#[test]
fn trade_bad_item() {
    let mut s = teststate();
    assert_eq!(s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Gundla, item: Item::Key }), Err(CommandError::InvalidItemError(Item::Key)));
}

#[test]
fn trade_bad_return_item() {
    let mut s = teststate();
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Gundla, item: Item::BagKey }).unwrap();
    assert_eq!(s.apply_command(Player::Gundla, Command::AcceptTrade { item: Item::BagKey }), Err(CommandError::InvalidItemError(Item::BagKey)));
}

#[test]
fn trade_bag_invalid() {
    let mut s = teststate();
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Gundla, item: Item::BagKey }).unwrap();
    assert_eq!(s.apply_command(Player::Gundla, Command::AcceptTrade { item: Item::BagGoblet }), Err(CommandError::InvalidItemError(Item::BagGoblet)));
}


#[test]
fn trade_bag_reject() {
    let mut s = teststate();
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Gundla, item: Item::BagKey }).unwrap();
    s.apply_command(Player::Gundla, Command::RejectTrade).unwrap();
}


#[test]
fn trade_bag_valid() {
    let mut s = teststate();
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::BagKey }).unwrap();
    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::PoisonRing }).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    assert!(s.game.p.player(Player::Sarah).items.contains(&Item::PoisonRing));
    assert!(!s.game.p.player(Player::Sarah).items.contains(&Item::BagKey));
    assert!(s.game.p.player(Player::Sarah).items.contains(&Item::Dagger)); // test that the effect triggered

    assert!(s.game.p.player(Player::Marie).items.contains(&Item::BagKey));
    assert!(!s.game.p.player(Player::Marie).items.contains(&Item::PoisonRing));
}

#[test]
fn trade_blackpearl_no_reject() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::BlackPearl);
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::BlackPearl }).unwrap();
    assert_eq!(s.apply_command(Player::Marie, Command::RejectTrade), Err(CommandError::MustAccept));
    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::PoisonRing }).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    assert!(s.game.p.player(Player::Sarah).items.contains(&Item::PoisonRing));
    assert!(!s.game.p.player(Player::Sarah).items.contains(&Item::BlackPearl));

    assert!(s.game.p.player(Player::Marie).items.contains(&Item::BlackPearl));
    assert!(!s.game.p.player(Player::Marie).items.contains(&Item::PoisonRing));
}

#[test]
fn trade_brokenmirror_no_reject() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::BrokenMirror);
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::BrokenMirror }).unwrap();
    assert_eq!(s.apply_command(Player::Marie, Command::RejectTrade), Err(CommandError::MustAccept));
    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::PoisonRing }).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    assert!(s.game.p.player(Player::Sarah).items.contains(&Item::PoisonRing));
    assert!(!s.game.p.player(Player::Sarah).items.contains(&Item::BrokenMirror));

    assert!(s.game.p.player(Player::Marie).items.contains(&Item::BrokenMirror));
    assert!(!s.game.p.player(Player::Marie).items.contains(&Item::PoisonRing));
}

#[test]
fn trade_monocle() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::Monocle);
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::Monocle }).unwrap();
    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::PoisonRing }).unwrap();
    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: true, trigger: PerspectiveTradeTriggerState::Monocle { faction: Some(Faction::Order) } });
    assert_eq!(s.perspective(Player::Zacharias).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: true, trigger: PerspectiveTradeTriggerState::Monocle { faction: None } });
    s.apply_command(Player::Sarah, Command::DoneLookingAtThings).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
}

#[test]
fn trade_monocle_backwards() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::Monocle);
    s.turn = TurnState::WaitingForQuickblink(Player::Marie);
    s.apply_command(Player::Marie, Command::OfferTrade { target: Player::Sarah, item: Item::PoisonRing }).unwrap();
    s.apply_command(Player::Sarah, Command::AcceptTrade { item: Item::Monocle }).unwrap();
    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Marie, target: Player::Sarah, is_first_item: false, trigger: PerspectiveTradeTriggerState::Monocle { faction: Some(Faction::Order) } });
    assert_eq!(s.perspective(Player::Zacharias).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Marie, target: Player::Sarah, is_first_item: false, trigger: PerspectiveTradeTriggerState::Monocle { faction: None } });
    s.apply_command(Player::Sarah, Command::DoneLookingAtThings).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Zacharias));
}

#[test]
fn trade_monocle_priviledge() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::Monocle);
    s.game.p.player_mut(Player::Marie).items.push(Item::Priviledge);
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::Monocle }).unwrap();
    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::Priviledge }).unwrap();
    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: true, trigger: PerspectiveTradeTriggerState::Monocle { faction: Some(Faction::Order) } });
    assert_eq!(s.perspective(Player::Zacharias).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: true, trigger: PerspectiveTradeTriggerState::Monocle { faction: None } });
    s.apply_command(Player::Sarah, Command::DoneLookingAtThings).unwrap();
    assert_eq!(s.perspective(Player::Marie).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: false, trigger: PerspectiveTradeTriggerState::Priviledge { items: Some(vec![Item::BagKey, Item::Priviledge]) } });
    assert_eq!(s.perspective(Player::Zacharias).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: false, trigger: PerspectiveTradeTriggerState::Priviledge { items: None } });
    s.apply_command(Player::Marie, Command::DoneLookingAtThings).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
}

#[test]
fn trade_monocle_with_bag_causing_donation() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Marie).items.push(Item::Monocle);
    s.game.p.player_mut(Player::Sarah).items.extend_from_slice(&[Item::BagGoblet, Item::Key, Item::Key, Item::Key, Item::Goblet]);

    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::BagGoblet }).unwrap();
    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::Monocle }).unwrap();
    s.apply_command(Player::Sarah, Command::DonateItem { target: Player::Zacharias, item: Item::Key }).unwrap();
    assert_eq!(s.perspective(Player::Marie).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: false, trigger: PerspectiveTradeTriggerState::Monocle { faction: Some(Faction::Order) } });
    assert_eq!(s.perspective(Player::Zacharias).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: false, trigger: PerspectiveTradeTriggerState::Monocle { faction: None } });
    s.apply_command(Player::Marie, Command::DoneLookingAtThings).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
}

#[test]
fn trade_priviledge() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::Priviledge);
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::Priviledge }).unwrap();
    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::PoisonRing }).unwrap();
    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: true, trigger: PerspectiveTradeTriggerState::Priviledge { items: Some(vec![Item::Priviledge]) } });
    assert_eq!(s.perspective(Player::Zacharias).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: true, trigger: PerspectiveTradeTriggerState::Priviledge { items: None } });
    s.apply_command(Player::Sarah, Command::DoneLookingAtThings).unwrap();
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
}

#[test]
fn trade_tome() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::Tome);
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::Tome }).unwrap();

    assert_eq!(s.game.p.player(Player::Sarah).job, Job::Duelist);
    assert_eq!(s.game.p.player(Player::Marie).job, Job::Thug);

    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::PoisonRing }).unwrap();

    assert_eq!(s.game.p.player(Player::Sarah).job, Job::Thug);
    assert_eq!(s.game.p.player(Player::Marie).job, Job::Duelist);

    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
}

#[test]
fn trade_sextant() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::Sextant);
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::Sextant }).unwrap();
    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::PoisonRing }).unwrap();

    s.apply_command(Player::Sarah, Command::SetSextantDirection { forward: true }).unwrap();

    s.apply_command(Player::Zacharias, Command::SelectSextantItem { item: Item::Gloves }).unwrap();
    s.apply_command(Player::Gundla, Command::SelectSextantItem { item: Item::BagGoblet }).unwrap();
    s.apply_command(Player::Sarah, Command::SelectSextantItem { item: Item::BagKey }).unwrap();
    s.apply_command(Player::Marie, Command::SelectSextantItem { item: Item::Sextant }).unwrap();

    assert_eq!(s.game.p.player(Player::Zacharias).items, vec![Item::Sextant]);
    assert_eq!(s.game.p.player(Player::Sarah).items, vec![Item::PoisonRing, Item::Gloves]);
    assert_eq!(s.game.p.player(Player::Gundla).items, vec![Item::BagKey]);
    assert_eq!(s.game.p.player(Player::Marie).items, vec![Item::BagGoblet]);
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
}

#[test]
fn trade_coat() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::Coat);
    s.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Marie, item: Item::Coat }).unwrap();
    s.apply_command(Player::Marie, Command::AcceptTrade { item: Item::PoisonRing }).unwrap();

    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: true, trigger: PerspectiveTradeTriggerState::Coat { available_jobs: Some(s.game.job_stack.clone()) } });
    assert_eq!(s.perspective(Player::Zacharias).turn, PerspectiveTurnState::ResolvingTradeTrigger { offerer: Player::Sarah, target: Player::Marie, is_first_item: true, trigger: PerspectiveTradeTriggerState::Coat { available_jobs: None } });

    s.apply_command(Player::Sarah, Command::PickNewJob { job: Job::Doctor }).unwrap();
    assert_eq!(s.game.p.player(Player::Sarah).job, Job::Doctor);
    assert!(!s.game.p.player(Player::Sarah).job_is_visible);
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
}

#[test]
fn victory_solo_bad() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.extend_from_slice(&[Item::Key, Item::Key]);
    s.apply_command(Player::Sarah, Command::AnnounceVictory { teammates: Vec::new() }).unwrap();
    assert_eq!(s.turn, TurnState::GameOver { winner: Faction::Brotherhood });
}

#[test]
fn victory_solo_good() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.extend_from_slice(&[Item::Key, Item::Key, Item::Key]);
    s.apply_command(Player::Sarah, Command::AnnounceVictory { teammates: Vec::new() }).unwrap();
    assert_eq!(s.turn, TurnState::GameOver { winner: Faction::Order });
}

#[test]
fn victory_team_bad() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.extend_from_slice(&[Item::Key]);
    s.game.p.player_mut(Player::Gundla).items.extend_from_slice(&[Item::Key, Item::Key]);
    s.apply_command(Player::Sarah, Command::AnnounceVictory { teammates: vec![Player::Gundla] }).unwrap();
    assert_eq!(s.turn, TurnState::GameOver { winner: Faction::Brotherhood });
}

#[test]
fn victory_team_bad2() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.extend_from_slice(&[Item::Key]);
    s.game.p.player_mut(Player::Marie).items.extend_from_slice(&[Item::Key]);
    s.apply_command(Player::Sarah, Command::AnnounceVictory { teammates: vec![Player::Marie] }).unwrap();
    assert_eq!(s.turn, TurnState::GameOver { winner: Faction::Brotherhood });
}

#[test]
fn victory_team_good() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.extend_from_slice(&[Item::Key]);
    s.game.p.player_mut(Player::Marie).items.extend_from_slice(&[Item::Key, Item::Key]);
    s.apply_command(Player::Sarah, Command::AnnounceVictory { teammates: vec![Player::Marie] }).unwrap();
    assert_eq!(s.turn, TurnState::GameOver { winner: Faction::Order });
}

#[test]
fn victory_black_pearl() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.extend_from_slice(&[Item::Key, Item::Key, Item::Key, Item::BlackPearl]);
    assert_eq!(s.apply_command(Player::Sarah, Command::AnnounceVictory { teammates: Vec::new() }), Err(CommandError::BlackPearl));
}

#[test]
fn attack_tie() {
    let mut s = teststate();
    s.apply_command(Player::Sarah, Command::InitiateAttack { player: Player::Zacharias }).unwrap();

    s.apply_command(Player::Sarah, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Gundla, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Marie, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Zacharias, Command::UsePriest { priest: false }).unwrap();

    s.apply_command(Player::Gundla, Command::DeclareSupport { support: AttackSupport::Attack }).unwrap();
    s.apply_command(Player::Marie, Command::DeclareSupport { support: AttackSupport::Defend }).unwrap();

    s.apply_command(Player::Sarah, Command::Hypnotize { target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Gundla, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Marie, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Zacharias, Command::ItemOrJob { buff: None, target: None }).unwrap();

    assert_eq!(s.game.p.player(Player::Sarah).items, vec![Item::BagKey, Item::Dagger]);
    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
}

#[test]
fn attack_win_creds() {
    let mut s = teststate();
    s.apply_command(Player::Sarah, Command::InitiateAttack { player: Player::Zacharias }).unwrap();

    s.apply_command(Player::Sarah, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Gundla, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Marie, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Zacharias, Command::UsePriest { priest: false }).unwrap();

    s.apply_command(Player::Gundla, Command::DeclareSupport { support: AttackSupport::Attack }).unwrap();
    s.apply_command(Player::Marie, Command::DeclareSupport { support: AttackSupport::Defend }).unwrap();

    s.apply_command(Player::Sarah, Command::Hypnotize { target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: Some(BuffSource::Job(Job::Duelist)), target: None }).unwrap();
    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Gundla, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Marie, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Zacharias, Command::ItemOrJob { buff: None, target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ClaimReward { steal_items: false }).unwrap();
    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::FinishResolvingCredentials { target_faction: Faction::Brotherhood, target_job: Job::Hypnotist } });
    assert_eq!(s.perspective(Player::Gundla).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::Normal(AttackState::FinishResolving { winner: AttackWinner::Attacker, steal_items: false }) });
    s.apply_command(Player::Sarah, Command::DoneLookingAtThings).unwrap();

    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    assert_eq!(s.game.p.player(Player::Sarah).items, vec![Item::BagKey]);
}

#[test]
fn attack_win_steal_giveback() {
    let mut s = teststate();
    s.apply_command(Player::Sarah, Command::InitiateAttack { player: Player::Zacharias }).unwrap();

    s.apply_command(Player::Sarah, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Gundla, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Marie, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Zacharias, Command::UsePriest { priest: false }).unwrap();

    s.apply_command(Player::Gundla, Command::DeclareSupport { support: AttackSupport::Attack }).unwrap();
    s.apply_command(Player::Marie, Command::DeclareSupport { support: AttackSupport::Defend }).unwrap();

    s.apply_command(Player::Sarah, Command::Hypnotize { target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: Some(BuffSource::Job(Job::Duelist)), target: None }).unwrap();
    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Gundla, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Marie, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Zacharias, Command::ItemOrJob { buff: None, target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ClaimReward { steal_items: true }).unwrap();
    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::FinishResolvingItems { target_items: vec![Item::Gloves] } });
    assert_eq!(s.perspective(Player::Gundla).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::Normal(AttackState::FinishResolving { winner: AttackWinner::Attacker, steal_items: true }) });
    s.apply_command(Player::Sarah, Command::StealItem { item: Item::Gloves, give_back: Some(Item::BagKey) }).unwrap();

    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    assert_eq!(s.game.p.player(Player::Sarah).items, vec![Item::Gloves]);
    assert_eq!(s.game.p.player(Player::Zacharias).items, vec![Item::BagKey]);
}

#[test]
fn attack_win_steal() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Zacharias).items.push(Item::Coat);

    s.apply_command(Player::Sarah, Command::InitiateAttack { player: Player::Zacharias }).unwrap();

    s.apply_command(Player::Sarah, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Gundla, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Marie, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Zacharias, Command::UsePriest { priest: false }).unwrap();

    s.apply_command(Player::Gundla, Command::DeclareSupport { support: AttackSupport::Attack }).unwrap();
    s.apply_command(Player::Marie, Command::DeclareSupport { support: AttackSupport::Defend }).unwrap();

    s.apply_command(Player::Sarah, Command::Hypnotize { target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: Some(BuffSource::Job(Job::Duelist)), target: None }).unwrap();
    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Gundla, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Marie, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Zacharias, Command::ItemOrJob { buff: None, target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ClaimReward { steal_items: true }).unwrap();
    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::FinishResolvingItems { target_items: vec![Item::Gloves, Item::Coat] } });
    assert_eq!(s.perspective(Player::Gundla).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::Normal(AttackState::FinishResolving { winner: AttackWinner::Attacker, steal_items: true }) });
    s.apply_command(Player::Sarah, Command::StealItem { item: Item::Gloves, give_back: None }).unwrap();

    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    assert_eq!(s.game.p.player(Player::Sarah).items, vec![Item::BagKey, Item::Gloves]);
    assert_eq!(s.game.p.player(Player::Zacharias).items, vec![Item::Coat]);
}

#[test]
fn attack_win_steal_donate() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Zacharias).items.push(Item::Coat);
    s.game.p.player_mut(Player::Sarah).items.extend_from_slice(&[Item::Key, Item::Key, Item::Key, Item::Goblet, Item::Goblet]);

    s.apply_command(Player::Sarah, Command::InitiateAttack { player: Player::Zacharias }).unwrap();

    s.apply_command(Player::Sarah, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Gundla, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Marie, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Zacharias, Command::UsePriest { priest: false }).unwrap();

    s.apply_command(Player::Gundla, Command::DeclareSupport { support: AttackSupport::Attack }).unwrap();
    s.apply_command(Player::Marie, Command::DeclareSupport { support: AttackSupport::Defend }).unwrap();

    s.apply_command(Player::Sarah, Command::Hypnotize { target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: Some(BuffSource::Job(Job::Duelist)), target: None }).unwrap();
    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Gundla, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Marie, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Zacharias, Command::ItemOrJob { buff: None, target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ClaimReward { steal_items: true }).unwrap();
    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::FinishResolvingItems { target_items: vec![Item::Gloves, Item::Coat] } });
    assert_eq!(s.perspective(Player::Gundla).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::Normal(AttackState::FinishResolving { winner: AttackWinner::Attacker, steal_items: true }) });
    s.apply_command(Player::Sarah, Command::StealItem { item: Item::Gloves, give_back: None }).unwrap();

    s.apply_command(Player::Sarah, Command::DonateItem { target: Player::Marie, item: Item::Key }).unwrap();

    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    assert_eq!(s.game.p.player(Player::Sarah).items, vec![Item::BagKey, Item::Key, Item::Key, Item::Goblet, Item::Goblet, Item::Gloves]);
    assert_eq!(s.game.p.player(Player::Zacharias).items, vec![Item::Coat]);
    assert_eq!(s.game.p.player(Player::Marie).items, vec![Item::PoisonRing, Item::Key]);
}

#[test]
fn attack_win_steal_donate2() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Zacharias).items.push(Item::Coat);
    s.game.p.player_mut(Player::Sarah).items.extend_from_slice(&[Item::Key, Item::Key, Item::Key, Item::Goblet, Item::Goblet]);

    s.apply_command(Player::Sarah, Command::InitiateAttack { player: Player::Zacharias }).unwrap();

    s.apply_command(Player::Sarah, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Gundla, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Marie, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Zacharias, Command::UsePriest { priest: false }).unwrap();

    s.apply_command(Player::Gundla, Command::DeclareSupport { support: AttackSupport::Attack }).unwrap();
    s.apply_command(Player::Marie, Command::DeclareSupport { support: AttackSupport::Defend }).unwrap();

    s.apply_command(Player::Sarah, Command::Hypnotize { target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: Some(BuffSource::Job(Job::Duelist)), target: None }).unwrap();
    s.apply_command(Player::Sarah, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Gundla, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Marie, Command::ItemOrJob { buff: None, target: None }).unwrap();
    s.apply_command(Player::Zacharias, Command::ItemOrJob { buff: None, target: None }).unwrap();

    s.apply_command(Player::Sarah, Command::ClaimReward { steal_items: true }).unwrap();
    assert_eq!(s.perspective(Player::Sarah).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::FinishResolvingItems { target_items: vec![Item::Gloves, Item::Coat] } });
    assert_eq!(s.perspective(Player::Gundla).turn, PerspectiveTurnState::Attacking { attacker: Player::Sarah, defender: Player::Zacharias, state: PerspectiveAttackState::Normal(AttackState::FinishResolving { winner: AttackWinner::Attacker, steal_items: true }) });
    s.apply_command(Player::Sarah, Command::StealItem { item: Item::Gloves, give_back: None }).unwrap();

    s.apply_command(Player::Sarah, Command::DonateItem { target: Player::Marie, item: Item::Gloves }).unwrap();

    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    assert_eq!(s.game.p.player(Player::Sarah).items, vec![Item::BagKey, Item::Key, Item::Key, Item::Key, Item::Goblet, Item::Goblet]);
    assert_eq!(s.game.p.player(Player::Zacharias).items, vec![Item::Coat]);
    assert_eq!(s.game.p.player(Player::Marie).items, vec![Item::PoisonRing, Item::Gloves]);
}

#[test]
fn attack_priest() {
    let mut s = teststate();
    s.game.p.player_mut(Player::Sarah).items.push(Item::Coat);
    s.game.p.player_mut(Player::Zacharias).job = Job::Priest;

    s.apply_command(Player::Sarah, Command::InitiateAttack { player: Player::Zacharias }).unwrap();

    s.apply_command(Player::Sarah, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Gundla, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Marie, Command::UsePriest { priest: false }).unwrap();
    s.apply_command(Player::Zacharias, Command::UsePriest { priest: true }).unwrap();

    s.apply_command(Player::Sarah, Command::PayPriest { item: Item::Coat }).unwrap();

    assert_eq!(s.turn, TurnState::WaitingForQuickblink(Player::Gundla));
    assert_eq!(s.game.p.player(Player::Sarah).items, vec![Item::BagKey]);
    assert_eq!(s.game.p.player(Player::Zacharias).items, vec![Item::Gloves, Item::Coat]);
}
