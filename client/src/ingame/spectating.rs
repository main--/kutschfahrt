use web_protocol::{
    ActionLogEntry, AttackState, Faction, PerspectiveAttackState, PerspectivePlayer,
    PerspectiveTurnState, SpectatorPerspective, WinningFaction,
};
use yew::{function_component, html, use_context, Html, Properties};

use super::{Lang, Translate, action_log_text};

#[derive(Properties, PartialEq)]
pub struct SpectatingProps {
    pub state: SpectatorPerspective,
}

#[function_component(Spectating)]
pub fn spectating(SpectatingProps { state }: &SpectatingProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    html! {
        <div class="hud spectating">
            <div class="spectating-badge">{lang.spectating()}</div>
            <SpectatorPlayerList players={state.players.clone()} item_stack={state.item_stack} />
            <SpectatorTurnInfo turn={state.turn.clone()} />
            <SpectatorActionLog log={state.action_log.clone()} />
        </div>
    }
}

// ── Player list ───────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct SpectatorPlayerListProps {
    players: Vec<PerspectivePlayer>,
    item_stack: usize,
}

#[function_component(SpectatorPlayerList)]
fn spectator_player_list(SpectatorPlayerListProps { players, item_stack }: &SpectatorPlayerListProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    html! {
        <div class="playerlist">
            <div class="entry head">
                <div>{lang.player_col()}</div>
                <div class="job">{lang.job_col()}</div>
                <div>{lang.items_col()}</div>
            </div>
            {for players.iter().map(|p| html! {
                <div class="entry">
                    <div class="name">{p.player.to_string()}</div>
                    <div class="job" data-tooltip={p.job.map(|j| j.tr_tooltip(lang)).unwrap_or_default()}>
                        {p.job.map(|j| j.tr_name(lang).to_string()).unwrap_or_else(|| "?".to_owned())}
                    </div>
                    <div class="item_count">{p.item_count}</div>
                </div>
            })}
            <div class="entry">
                <div>{lang.draw_pile()}</div>
                <div class="job">{""}</div>
                <div class="item_count">{item_stack}</div>
            </div>
        </div>
    }
}

// ── Turn info ─────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct SpectatorTurnInfoProps {
    turn: PerspectiveTurnState,
}

#[function_component(SpectatorTurnInfo)]
fn spectator_turn_info(SpectatorTurnInfoProps { turn }: &SpectatorTurnInfoProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    let text = match turn {
        PerspectiveTurnState::TurnStart { player } =>
            lang.spec_turn(&player.to_string()),
        PerspectiveTurnState::TurnEndPhase { player } =>
            lang.spec_ending_turn(&player.to_string()),
        PerspectiveTurnState::DonatingItem { donor } =>
            lang.spec_donating(&donor.to_string()),
        PerspectiveTurnState::TradePending { offerer, target, .. } =>
            lang.spec_trade(&offerer.to_string(), &target.to_string()),
        PerspectiveTurnState::ResolvingTradeTrigger { giver, receiver, .. } =>
            lang.spec_trigger(&giver.to_string(), &receiver.to_string()),
        PerspectiveTurnState::DoingClairvoyant { player, .. } =>
            lang.spec_clairvoyant(&player.to_string()),
        PerspectiveTurnState::UnsuccessfulDiplomat { diplomat, target, .. } =>
            lang.spec_diplomat(&diplomat.to_string(), &target.to_string()),
        PerspectiveTurnState::GameOver { winner: WinningFaction::Normal(Faction::Order) } =>
            lang.spec_gameover_order().to_string(),
        PerspectiveTurnState::GameOver { winner: WinningFaction::Normal(Faction::Brotherhood) } =>
            lang.spec_gameover_brotherhood().to_string(),
        PerspectiveTurnState::GameOver { winner: WinningFaction::Traitor(player) } =>
            lang.spec_gameover_traitor(&player.to_string()),
        PerspectiveTurnState::Attacking { attacker, defender, state } =>
            format!("{} – {}", lang.is_attacking(&attacker.to_string(), &defender.to_string()),
                    attack_phase_text(state, lang)),
    };

    html! {
        <div class="spectating-turn-info">
            <p>{ text }</p>
        </div>
    }
}

fn attack_phase_text(state: &PerspectiveAttackState, lang: Lang) -> &'static str {
    match state {
        PerspectiveAttackState::Normal(AttackState::WaitingForPriest { .. }) =>
            lang.spec_attack_waiting_priest(),
        PerspectiveAttackState::Normal(AttackState::PayingPriest { .. }) =>
            lang.spec_attack_priest_stopped(),
        PerspectiveAttackState::Normal(AttackState::DeclaringSupport(_)) =>
            lang.spec_attack_support(),
        PerspectiveAttackState::Normal(AttackState::WaitingForHypnotizer(_)) =>
            lang.spec_attack_hypnotist(),
        PerspectiveAttackState::Normal(AttackState::ItemsOrJobs { .. }) =>
            lang.spec_attack_items(),
        PerspectiveAttackState::Normal(AttackState::Resolving { .. }) =>
            lang.spec_attack_resolving(),
        PerspectiveAttackState::Normal(AttackState::FinishResolving { .. }) |
        PerspectiveAttackState::FinishResolvingNeedFactionIndex |
        PerspectiveAttackState::FinishResolvingCredentials { .. } |
        PerspectiveAttackState::FinishResolvingItems { .. } =>
            lang.spec_attack_reward(),
    }
}

// ── Action log ────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
struct SpectatorActionLogProps {
    log: Vec<ActionLogEntry>,
}

#[function_component(SpectatorActionLog)]
fn spectator_action_log(SpectatorActionLogProps { log }: &SpectatorActionLogProps) -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();
    html! {
        <div class="actionlog">
            {for log.iter().map(|action| {
                let body = action_log_text(action, lang);
                html! { <div class="entry">{ body }</div> }
            })}
        </div>
    }
}
