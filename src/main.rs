#![allow(dead_code)]

use rand::prelude::*;

use kutschfahrt::*;
use web_protocol::*;

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
    state.apply_command(Player::Marie, Command::ItemOrJob { buff: None, target: None }).unwrap();
    state.apply_command(Player::Gundla, Command::ItemOrJob { buff: None, target: None }).unwrap();
    state.apply_command(Player::Zacharias, Command::ItemOrJob { buff: Some(BuffSource::Item(Item::PoisonRing)), target: None }).unwrap();
    state.apply_command(Player::Sarah, Command::ItemOrJob { buff: None, target: None }).unwrap();
    state.apply_command(Player::Zacharias, Command::ItemOrJob { buff: None, target: None }).unwrap();
    state.apply_command(Player::Marie, Command::ItemOrJob { buff: None, target: None }).unwrap();
    state.apply_command(Player::Gundla, Command::ItemOrJob { buff: None, target: None }).unwrap();
    
    state.apply_command(Player::Zacharias, Command::ClaimReward { steal_items: false }).unwrap();
    state.apply_command(Player::Zacharias, Command::DoneLookingAtThings).unwrap();

    state.apply_command(Player::Gundla, Command::OfferTrade { target: Player::Sarah, item: Item::BagKey }).unwrap();
    state.apply_command(Player::Sarah, Command::RejectTrade).unwrap();

    state.apply_command(Player::Sarah, Command::OfferTrade { target: Player::Zacharias, item: Item::BagGoblet }).unwrap();
    state.apply_command(Player::Zacharias, Command::AcceptTrade { returned: Item::PoisonRing }).unwrap();

    state.apply_command(Player::Marie, Command::AnnounceVictory { teammates: vec![] }).unwrap();

    assert_eq!(state.turn, TurnState::GameOver { winner: Faction::Brotherhood });
}

