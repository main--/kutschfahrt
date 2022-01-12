use std::collections::HashMap;
use std::rc::Rc;

use web_protocol::{PerspectiveAttackState, Player, AttackState, Command, Job, Item, AttackSupport, Perspective, AttackWinner, inventory_limit, Buff, BuffScore};
use yew::prelude::*;
use super::{CommandButton, DoneLookingBtn, SelectItem, ItemListEntry};

#[derive(Properties, PartialEq)]
pub struct AttackingProps {
    pub p: Perspective,
    pub myself: Player,
    pub attacker: Player,
    pub defender: Player,
    pub state: PerspectiveAttackState,
}
#[function_component(Attacking)]
pub fn attacking(props: &AttackingProps) -> Html {
    let &AttackingProps { myself, attacker, defender, ref p, ref state } = props;
    let opponent = if attacker == myself { defender } else { attacker };

    let me = &p.you;



    let body = match state {
        PerspectiveAttackState::Normal(AttackState::WaitingForPriest { passed }) if passed.contains(&myself) => html! { format!("Waiting for other players to use Priest ...") },
        PerspectiveAttackState::Normal(AttackState::WaitingForPriest { .. }) => html! {
            <>
                {"Use priest?"}
                <CommandButton text={"Use"} command={if me.job == Job::Priest && !me.job_is_visible { Some(Command::UsePriest { priest: true }) } else { None }} />
                <CommandButton text={"Don't"} command={Some(Command::UsePriest { priest: false })} />
            </>
        },
        &PerspectiveAttackState::Normal(AttackState::PayingPriest { priest }) if myself == attacker => html! { <PayingPriest {priest} /> },
        &PerspectiveAttackState::Normal(AttackState::PayingPriest { priest }) => html! { {format!("Waiting for {} to give an item to the Priest ({}) ...", attacker, priest)} },

        &PerspectiveAttackState::FinishResolvingCredentials { target_faction, target_job } => html! {
            <>
                <p class="attack-text">{format!("You see that {}'s faction is {:?} and their job is {:?}.", opponent, target_faction, target_job)}</p>
                <DoneLookingBtn />
            </>
        },
        PerspectiveAttackState::FinishResolvingItems { target_items } => html! { <StealItems target_items={target_items.clone()} victim={opponent} /> },

        PerspectiveAttackState::Normal(AttackState::DeclaringSupport(support)) => {
            let all_players = p.players.iter().map(|p| p.player);
            let players_twice = all_players.clone().chain(all_players);
            let mut attack_supporters = players_twice
                .skip_while(|&x| x != attacker)
                .take(p.players.len())
                .filter(|&x| x != attacker && x != defender);

            let my_turn = attack_supporters.next() == Some(myself);

            html! {
                <>
                    <h3>{"Declaring support"}</h3>
                    <AttackOverview {attacker} {defender} votes={support.clone()} />
                    {if my_turn {
                        html! {
                            <div>
                                <CommandButton text={"Support Attacker"} command={Some(Command::DeclareSupport { support: AttackSupport::Attack })} />
                                <CommandButton text={"Support Defender"} command={Some(Command::DeclareSupport { support: AttackSupport::Defend })} />
                                <CommandButton text={"Abstain"} command={Some(Command::DeclareSupport { support: AttackSupport::Abstain })} />
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </>
            }
        }
        PerspectiveAttackState::Normal(AttackState::WaitingForHypnotizer(support)) => {
            let my_turn = myself == attacker;
            let am_hypnotist = p.you.job == Job::Hypnotist;

            html! {
                <>
                    <h3>{"Hypnotizing"}</h3>
                    <AttackOverview {attacker} {defender} votes={support.clone()} hypnotize_btn={am_hypnotist} />

                    {if my_turn {
                        html! {
                            <CommandButton text={"Don't hypnotize"} command={Some(Command::Hypnotize { target: None })} />
                        }
                    } else {
                        html! { <p>{"Waiting for hypnotizer ..."}</p> }
                    }}
                </>
            }
        }
        PerspectiveAttackState::Normal(AttackState::ItemsOrJobs { votes: support, buffs, passed }) => {
            let my_turn = !passed.contains(&myself);

            html! {
                <>
                    <h3>{"Items & Jobs"}</h3>
                    <AttackOverview {attacker} {defender} votes={support.clone()} buffs={buffs.clone()} />

                    {if my_turn {
                        html! {
                            <CommandButton text={"Pass"} command={Some(Command::ItemOrJob { buff: None, target: None })} />
                        }
                    } else {
                        html! { <p>{"You have passed."}</p> }
                    }}
                </>
            }
        }
        PerspectiveAttackState::Normal(AttackState::Resolving { winner }) => {
            let winner = match winner {
                AttackWinner::Attacker => attacker,
                AttackWinner::Defender => defender,
            };

            if winner == myself {
                html! {
                    <>
                        <CommandButton text={"Truth (Faction & Job)"} command={Some(Command::ClaimReward { steal_items: false })} />
                        <CommandButton text={"Items"} command={Some(Command::ClaimReward { steal_items: true })} />
                    </>
                }
            } else {
                html! { <p>{format!("Waiting for {} to claim a reward ...", winner)}</p> }
            }
        }
        &PerspectiveAttackState::Normal(AttackState::FinishResolving { winner, steal_items }) => {
            let winner = match winner {
                AttackWinner::Attacker => attacker,
                AttackWinner::Defender => defender,
            };

            if steal_items {
                html! { <p>{format!("Waiting for {} to steal items ...", winner)}</p> }
            } else {
                html! { <p>{format!("Waiting for {} to look at things ...", winner)}</p> }
            }
        }
    };

    html! {
        <>
            <p class="attack-text">{format!("{} is attacking {}", props.attacker, props.defender)}</p>
            {body}
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct PayingPriestProps {
    priest: Player,
}
#[function_component(PayingPriest)]
pub fn paying_priest(props: &PayingPriestProps) -> Html {
    let perspective = use_context::<Rc<Perspective>>().unwrap();

    let item = use_state(|| None);

    html! {
        <>
            <p>{format!("Select an item to give to the priest ({})", props.priest)}</p>
            <SelectItem on_change={Callback::from({ let item = item.clone(); move |i| item.set(i) })}>
                {for perspective.you.items.iter().map(|&i| html_nested! { <ItemListEntry item={i} can_select={true} /> })}
            </SelectItem>
            <CommandButton command={item.map(move |item| Command::PayPriest { item })} text={"Submit"} />
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct StealItemsProps {
    victim: Player,
    target_items: Vec<Item>,
}
#[function_component(StealItems)]
pub fn steal_items(props: &StealItemsProps) -> Html {
    let &StealItemsProps { victim, ref target_items } = props;
    let perspective = use_context::<Rc<Perspective>>().unwrap();

    let item = use_state(|| None);
    let giveback = use_state(|| None);
    let need_give_back = perspective.you.items.len() >= inventory_limit(perspective.players.len());

    html! {
        <>
            <p class="attack-text">{format!("Select an item to steal from {}.", victim)}</p>
            <SelectItem on_change={Callback::from({ let item = item.clone(); move |i| item.set(i) })}>
                {for target_items.iter().map(|&i| html_nested! { <ItemListEntry item={i} can_select={true} /> })}
            </SelectItem>
            {if need_give_back {
                html! {
                    <>
                        <p class="attack-text">{format!("Select an item to give back to {}.", victim)}</p>
                        <SelectItem on_change={Callback::from({ let giveback = giveback.clone(); move |i| giveback.set(i) })}>
                            {for perspective.you.items.iter().map(|&i| html_nested! { <ItemListEntry item={i} can_select={true} /> })}
                        </SelectItem>
                    </>
                }
            } else {
                html! {}
            }}
            <CommandButton command={item.map(move |item| Command::StealItem { item, give_back: *giveback })} text={"Submit"} />
        </>
    }
}


#[derive(Properties, PartialEq)]
pub struct AttackOverviewProps {
    attacker: Player,
    defender: Player,
    votes: HashMap<Player, AttackSupport>,
    #[prop_or_default]
    buffs: Vec<Buff>,
    #[prop_or_default]
    hypnotize_btn: bool,
}
#[function_component(AttackOverview)]
pub fn attack_overview(props: &AttackOverviewProps) -> Html {
    let &AttackOverviewProps { attacker, defender, ref votes, ref buffs, hypnotize_btn } = props;
    let p = use_context::<Rc<Perspective>>().unwrap();

    let all_players = p.players.iter().map(|p| p.player);
    let players_twice = all_players.clone().chain(all_players);
    let mut attack_supporters = players_twice
        .skip_while(|&x| x != attacker)
        .take(p.players.len())
        .filter(|&x| x != attacker && x != defender);

    let supporter_list: Vec<_> = attack_supporters.by_ref().take(votes.len()).collect();

    let (attv, defv): (Vec<_>, Vec<_>) = buffs.iter().map(|b| b.raw_score).chain(votes.values().map(|v| v.vote_value())).partition(|&v| v > 0);
    let attacking_votes = 2 + attv.into_iter().sum::<BuffScore>();
    let defending_votes = 2 + defv.into_iter().sum::<BuffScore>();

    html! {
        <>
            <ul>
                <li>{format!("{} is the attacker", attacker)}</li>
                {for supporter_list.into_iter().map(|p| html! {
                    <li>{format!("{}: {:?}", p, votes.get(&p).unwrap())} {if hypnotize_btn { html! { <CommandButton text={"Hypnotize"} command={Some(Command::Hypnotize { target: Some(p) })} /> } } else { html! {} }}</li>
                })}
                <li>{format!("{} is the defender", defender)}</li>
            </ul>
            {if buffs.is_empty() {
                html! {}
            } else {
                html! {
                    <>
                        <p>{"Active buffs:"}</p>
                        <ul>
                            {for buffs.iter().map(|buff| html! {
                                <li>{format!("{} uses {:?} ({})", buff.user, buff.source, buff_score(buff.raw_score))}</li>
                            })}
                        </ul>
                    </>
                }
            }}
            <p>{format!("Current tally: {} vs {}", buff_score(attacking_votes), buff_score(defending_votes))}</p>
        </>
    }
}

fn buff_score(x: BuffScore) -> String {
    format!("{}", (x as f32) / 2.)
}
