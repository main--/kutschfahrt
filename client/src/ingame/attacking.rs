use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use web_protocol::{PerspectiveAttackState, Player, AttackState, Command, Job, Item, AttackSupport, Perspective, AttackWinner, inventory_limit, Buff, BuffScore, BuffSource};
use yew::prelude::*;
use super::{CommandButton, DoneLookingBtn, SelectItem, ItemListEntry, Lang, Translate, faction_name};

#[derive(Properties, PartialEq)]
pub struct AttackingProps {
    pub myself: Player,
    pub attacker: Player,
    pub defender: Player,
    pub state: PerspectiveAttackState,
}
#[function_component(Attacking)]
pub fn attacking(props: &AttackingProps) -> Html {
    let &AttackingProps { myself, attacker, defender, ref state } = props;
    let opponent = if attacker == myself { defender } else { attacker };

    let p = use_context::<Rc<Perspective>>().unwrap();
    let me = &p.you;
    let lang = use_context::<Lang>().unwrap_or_default();

    let body = match state {
        PerspectiveAttackState::Normal(AttackState::WaitingForPriest { passed }) if passed.contains(&myself) => html! { <p>{lang.waiting_for_priest()}</p> },
        PerspectiveAttackState::Normal(AttackState::WaitingForPriest { .. }) => html! {
            <>
                <p>{lang.use_priest_prompt()}</p>
                <CommandButton text={lang.use_priest()} command={if me.job == Job::Priest && !me.job_is_visible { Some(Command::UsePriest { priest: true }) } else { None }} />
                <CommandButton text={lang.dont()} command={Some(Command::UsePriest { priest: false })} />
            </>
        },
        &PerspectiveAttackState::Normal(AttackState::PayingPriest { priest }) if myself == attacker => html! { <PayingPriest {priest} /> },
        &PerspectiveAttackState::Normal(AttackState::PayingPriest { priest }) => html! { <p>{lang.paying_priest_wait(&attacker.to_string(), &priest.to_string())}</p> },

        &PerspectiveAttackState::FinishResolvingCredentials { target_faction, target_job } => html! {
            <>
                <p class="attack-text">{lang.see_faction_job(&opponent.to_string(), faction_name(target_faction, lang), &target_job.tr_name(lang))}</p>
                <DoneLookingBtn />
            </>
        },
        PerspectiveAttackState::FinishResolvingItems { target_items } => html! { <StealItems target_items={target_items.clone()} victim={opponent} /> },
        PerspectiveAttackState::FinishResolvingNeedFactionIndex => {
            html! {
                <>
                    <p>{lang.pick_any_faction_card()}</p>
                    <CommandButton text={lang.card(1)} command={Some(Command::ThreePlayerSelectFactionIndex { index: 0 })} />
                    <CommandButton text={lang.card(2)} command={Some(Command::ThreePlayerSelectFactionIndex { index: 1 })} />
                    <CommandButton text={lang.card(3)} command={Some(Command::ThreePlayerSelectFactionIndex { index: 2 })} />
                </>
            }
        }

        PerspectiveAttackState::Normal(AttackState::DeclaringSupport(support)) => {
            let all_players = p.players.iter().map(|p| p.player);
            let players_twice = all_players.clone().chain(all_players);
            let mut attack_supporters = players_twice
                .skip_while(|&x| x != attacker)
                .take(p.players.len())
                .filter(|&x| x != attacker && x != defender);

            attack_supporters.by_ref().take(support.len()).count();

            let my_turn = attack_supporters.next() == Some(myself);

            html! {
                <>
                    <h3>{lang.declaring_support()}</h3>
                    <AttackOverview {attacker} {defender} votes={support.clone()} />
                    {if my_turn {
                        html! {
                            <div>
                                <CommandButton text={lang.support_attacker()} command={Some(Command::DeclareSupport { support: AttackSupport::Attack })} />
                                <CommandButton text={lang.support_defender()} command={Some(Command::DeclareSupport { support: AttackSupport::Defend })} />
                                <CommandButton text={lang.abstain()} command={Some(Command::DeclareSupport { support: AttackSupport::Abstain })} />
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </>
            }
        }
        PerspectiveAttackState::Normal(AttackState::WaitingForHypnotizer(votes)) => {
            let my_turn = myself == attacker;
            let am_hypnotist = p.you.job == Job::Hypnotist;

            html! {
                <>
                    <h3>{lang.hypnotizing()}</h3>
                    <AttackOverview {attacker} {defender} votes={votes.clone()} hypnotize_btn={am_hypnotist} />

                    {if my_turn {
                        html! {
                            <CommandButton text={lang.dont_hypnotize()} command={Some(Command::Hypnotize { target: None })} />
                        }
                    } else {
                        html! { <p>{lang.waiting_for_hypnotizer()}</p> }
                    }}
                </>
            }
        }
        PerspectiveAttackState::Normal(AttackState::ItemsOrJobs { votes, buffs, passed }) => html! { <ItemsAndJobs {attacker} {defender} votes={votes.clone()} buffs={buffs.clone()} passed={passed.clone()} />},
        PerspectiveAttackState::Normal(AttackState::Resolving { winner }) => {
            let winner = match winner {
                AttackWinner::Attacker => attacker,
                AttackWinner::Defender => defender,
            };

            if winner == myself {
                html! {
                    <>
                        <CommandButton text={lang.truth_reward()} command={Some(Command::ClaimReward { steal_items: false })} />
                        <CommandButton text={lang.items_reward()} command={Some(Command::ClaimReward { steal_items: true })} />
                    </>
                }
            } else {
                html! { <p>{lang.waiting_for_reward(&winner.to_string())}</p> }
            }
        }
        &PerspectiveAttackState::Normal(AttackState::FinishResolving { winner, steal_items, three_player_faction_index }) => {
            let winner = match winner {
                AttackWinner::Attacker => attacker,
                AttackWinner::Defender => defender,
            };

            if steal_items {
                html! { <p>{lang.waiting_for_steal(&winner.to_string())}</p> }
            } else if let Some(i) = three_player_faction_index {
                html! { <p>{lang.waiting_for_faction_look_n(&winner.to_string(), (i + 1) as usize)}</p> }
            } else {
                html! { <p>{lang.waiting_for_faction_look(&winner.to_string())}</p> }
            }
        }
    };

    html! {
        <>
            <p class="attack-text">{lang.is_attacking(&props.attacker.to_string(), &props.defender.to_string())}</p>
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
    let lang = use_context::<Lang>().unwrap_or_default();

    let item = use_state(|| None);

    html! {
        <>
            <p>{lang.select_item_for_priest(&props.priest.to_string())}</p>
            <SelectItem on_change={Callback::from({ let item = item.clone(); move |i| item.set(i) })}>
                {for perspective.you.items.iter().map(|&i| html_nested! { <ItemListEntry item={i} can_select={true} /> })}
            </SelectItem>
            <CommandButton command={item.map(move |item| Command::PayPriest { item })} text={lang.submit()} />
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
    let lang = use_context::<Lang>().unwrap_or_default();

    let item = use_state(|| None);
    let giveback = use_state(|| None);
    let need_give_back = perspective.you.items.len() >= inventory_limit(perspective.players.len());

    html! {
        <>
            <p class="attack-text">{lang.steal_item_from(&victim.to_string())}</p>
            <SelectItem on_change={Callback::from({ let item = item.clone(); move |i| item.set(i) })}>
                {for target_items.iter().map(|&i| html_nested! { <ItemListEntry item={i} can_select={true} /> })}
            </SelectItem>
            {if need_give_back {
                html! {
                    <>
                        <p class="attack-text">{lang.give_back_to(&victim.to_string())}</p>
                        <SelectItem on_change={Callback::from({ let giveback = giveback.clone(); move |i| giveback.set(i) })}>
                            {for perspective.you.items.iter().map(|&i| html_nested! { <ItemListEntry item={i} can_select={true} /> })}
                        </SelectItem>
                    </>
                }
            } else {
                html! {}
            }}
            <CommandButton command={item.map(move |item| Command::StealItem { item, give_back: *giveback })} text={lang.submit()} />
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
    let lang = use_context::<Lang>().unwrap_or_default();

    let all_players = p.players.iter().map(|p| p.player);
    let players_twice = all_players.clone().chain(all_players);
    let mut attack_supporters = players_twice
        .skip_while(|&x| x != attacker)
        .take(p.players.len())
        .filter(|&x| x != attacker && x != defender);

    let supporter_list: Vec<_> = attack_supporters.by_ref().take(votes.len()).collect();

    let (attv, defv): (Vec<_>, Vec<_>) = buffs.iter().map(|b| b.raw_score).chain(votes.values().map(|v| v.vote_value())).partition(|&v| v > 0);
    let attacking_votes = 2 + attv.into_iter().sum::<BuffScore>();
    let defending_votes = 2 - defv.into_iter().sum::<BuffScore>();

    html! {
        <>
            <ul>
                <li>{lang.is_attacker(&attacker.to_string())}</li>
                {for supporter_list.into_iter().map(|p| html! {
                    <li>{format!("{}: {:?}", p, votes.get(&p).unwrap())} {if hypnotize_btn { html! { <CommandButton text={lang.hypnotize()} command={Some(Command::Hypnotize { target: Some(p) })} /> } } else { html! {} }}</li>
                })}
                <li>{lang.is_defender(&defender.to_string())}</li>
            </ul>
            {if buffs.is_empty() {
                html! {}
            } else {
                html! {
                    <>
                        <p>{lang.active_buffs()}</p>
                        <ul>
                            {for buffs.iter().map(|buff| html! {
                                <li>{format!("{} {} ({})", buff.user, buff_source_name(&buff.source, lang), buff_score(buff.raw_score))}</li>
                            })}
                        </ul>
                    </>
                }
            }}
            <p>{lang.tally(&buff_score(attacking_votes), &buff_score(defending_votes))}</p>
        </>
    }
}

fn buff_score(x: BuffScore) -> String {
    format!("{}", (x as f32) / 2.)
}

fn buff_source_name(source: &BuffSource, lang: Lang) -> String {
    match source {
        BuffSource::Item(item) => format!("{} ({})", lang.use_item(), item.tr_name(lang)),
        BuffSource::Job(job)   => lang.use_job(&job.tr_name(lang)),
    }
}

#[derive(Properties, PartialEq)]
pub struct ItemsAndJobsProps {
    attacker: Player,
    defender: Player,
    votes: HashMap<Player, AttackSupport>,
    buffs: Vec<Buff>,
    passed: HashSet<Player>,
}
#[function_component(ItemsAndJobs)]
pub fn items_and_jobs(props: &ItemsAndJobsProps) -> Html {
    let &ItemsAndJobsProps { attacker, defender, ref votes, ref buffs, ref passed } = props;
    let p = use_context::<Rc<Perspective>>().unwrap();
    let lang = use_context::<Lang>().unwrap_or_default();

    let myself = p.players[p.your_player_index].player;
    let my_turn = !passed.contains(&myself);

    let item = use_state(|| None);

    html! {
        <>
            <h3>{lang.items_and_jobs()}</h3>
            <AttackOverview {attacker} {defender} votes={votes.clone()} buffs={buffs.clone()} />

            {if my_turn {
                html! {
                    <>
                        <SelectItem on_change={Callback::from({ let item = item.clone(); move |i| item.set(i) })}>
                            {for p.you.items.iter().map(|&i| html_nested! { <ItemListEntry item={i} can_select={true} /> })}
                        </SelectItem>
                        <CommandButton text={lang.use_item()} command={item.map(move |item| Command::ItemOrJob { buff: Some(BuffSource::Item(item)), target: None })} />

                        {if p.you.job == Job::PoisonMixer {
                            html! {
                                <>
                                    <CommandButton text={lang.make_attacker_win()} command={Some(Command::ItemOrJob { buff: Some(BuffSource::Job(p.you.job)), target: Some(attacker) })} />
                                    <CommandButton text={lang.make_defender_win()} command={Some(Command::ItemOrJob { buff: Some(BuffSource::Job(p.you.job)), target: Some(defender) })} />
                                </>
                            }
                        } else {
                            html! { <CommandButton text={lang.use_job(&p.you.job.tr_name(lang))} command={Some(Command::ItemOrJob { buff: Some(BuffSource::Job(p.you.job)), target: None })} /> }
                        }}

                        <CommandButton text={lang.pass_items()} command={Some(Command::ItemOrJob { buff: None, target: None })} />
                    </>
                }
            } else {
                html! { <p>{lang.you_passed()}</p> }
            }}
        </>
    }
}
