use web_protocol::{Item, Job};
use yew::{function_component, html, use_context, Html};

use crate::ingame::{Lang, Translate};

fn s(lang: Lang, en: &'static str, de: &'static str) -> &'static str {
    match lang { Lang::En => en, Lang::De => de }
}

const ALL_JOBS: &[Job] = &[
    Job::Thug, Job::GrandMaster, Job::Bodyguard, Job::Duelist,
    Job::PoisonMixer, Job::Doctor, Job::Priest, Job::Hypnotist,
    Job::Diplomat, Job::Clairvoyant,
];

fn item_emoji(item: Item) -> &'static str {
    match item {
        Item::Key                  => "🔑",
        Item::Goblet               => "🏆",
        Item::BagKey               => "🧳",
        Item::BagGoblet            => "🧳",
        Item::BlackPearl           => "🖤",
        Item::Dagger               => "🗡️",
        Item::Gloves               => "🧤",
        Item::PoisonRing           => "💍",
        Item::CastingKnives        => "🔪",
        Item::Whip                 => "🪢",
        Item::Priviledge           => "📜",
        Item::Monocle              => "🧐",
        Item::BrokenMirror         => "🪞",
        Item::Sextant              => "🧭",
        Item::Coat                 => "🧥",
        Item::Tome                 => "📖",
        Item::CoatOfArmorOfTheLoge => "🛡️",
    }
}

const ALL_ITEMS: &[Item] = &[
    Item::Key, Item::Goblet, Item::BagKey, Item::BagGoblet,
    Item::BlackPearl, Item::Dagger, Item::Gloves, Item::PoisonRing,
    Item::CastingKnives, Item::Whip, Item::Priviledge, Item::Monocle,
    Item::BrokenMirror, Item::Sextant, Item::Coat, Item::Tome,
    Item::CoatOfArmorOfTheLoge,
];

#[function_component(Rules)]
pub fn rules() -> Html {
    let lang = use_context::<Lang>().unwrap_or_default();

    html! {
        <div class="rules-page content">

            <h1 class="title is-2">
                {s(lang, "Game Rules", "Spielregeln")}
                <span class="rules-subtitle">{" - Kutschfahrt zur Teufelsburg"}</span>
            </h1>
            <p class="has-text-grey mb-5">
                {s(lang,
                    "Michael Palm & Lukas Zach - 3-10 players - Age 12+ - 30-60 min",
                    "Michael Palm & Lukas Zach - 3-10 Spieler - Ab 12 Jahren - 30-60 Min"
                )}
            </p>

            // ── Spielidee ─────────────────────────────────────────────────────
            <h2 class="title is-4 rules-section-title">{s(lang, "Idea of the Game", "Spielidee")}</h2>
            <div class="notification is-light">
                <p>{s(lang,
                    "All players belong to one of two secret societies and try to discover who their allies are during a coach ride to Devil's Castle. Secret objects change owners to gather information. The goal is to collect three objects of your faction together with your allies and proclaim victory.",
                    "Alle Spieler gehören einer von zwei geheimen Gesellschaften an und versuchen während einer Kutschfahrt herauszufinden, wer ihre Verbündeten sind. Geheime Gegenstände wechseln die Besitzer, um Informationen zu sammeln. Ziel ist es, gemeinsam mit den Verbündeten drei Gegenstände der eigenen Gesellschaft zu sammeln und den Sieg zu verkünden."
                )}</p>
            </div>

            // ── Spielablauf ───────────────────────────────────────────────────
            <h2 class="title is-4 rules-section-title">{s(lang, "Turn Structure", "Spielablauf")}</h2>
            <p>{s(lang,
                "The most widely-traveled player goes first; play proceeds clockwise. On your turn, choose one action:",
                "Der weitgereiste Spieler beginnt; es wird im Uhrzeigersinn weitergespielt. In deinem Zug wählst du eine Aktion:"
            )}</p>

            <div class="rules-actions">

                <div class="rules-action-card">
                    <p class="rules-action-title">{"A. "}{s(lang, "Pass", "Passen")}</p>
                    <p>{s(lang, "Do nothing.", "Nichts tun.")}</p>
                </div>

                <div class="rules-action-card">
                    <p class="rules-action-title">{"B. "}{s(lang, "Trade an Object", "Gegenstand tauschen")}</p>
                    <ol>
                        <li>{s(lang,
                            "Offer one of your objects face-down to any other player.",
                            "Biete einem anderen Spieler einen deiner Gegenstände verdeckt an."
                        )}</li>
                        <li>{s(lang,
                            "That player looks at it and decides to accept or refuse. They must refuse if their only object is one bag and the other bag is being offered.",
                            "Dieser schaut ihn an und entscheidet, ob er annimmt oder ablehnt. Er muss ablehnen, wenn sein einziger Gegenstand ein Koffer ist und der andere Koffer angeboten wird."
                        )}</li>
                        <li>{s(lang,
                            "If accepted: they keep the object and return a different one face-down. The offerer cannot refuse.",
                            "Bei Annahme: Er behält den Gegenstand und gibt einen anderen verdeckt zurück. Der Anbietende kann nicht ablehnen."
                        )}</li>
                        <li>{s(lang,
                            "If refused: the object is returned silently; no abilities activate.",
                            "Bei Ablehnung: Der Gegenstand wird still zurückgegeben; keine Fähigkeiten werden ausgelöst."
                        )}</li>
                    </ol>
                    <p class="mt-2 has-text-grey">{s(lang,
                        "Trade-trigger objects (text begins with 'Trade it in and...'): the receiver names and reads their card first, then the giver resolves theirs. Exception: Broken Mirror - both cards switch silently without being named.",
                        "Tausch-Gegenstände (Text beginnt mit 'Tauschst du ihn weiter...'): Der Empfänger nennt und liest seine Karte zuerst, dann löst der Geber seine aus. Ausnahme: Zerbrochener Spiegel - beide Karten wechseln still, ohne benannt zu werden."
                    )}</p>
                </div>

                <div class="rules-action-card">
                    <p class="rules-action-title">{"C. "}{s(lang, "Attack a Player", "Mitspieler angreifen")}</p>
                    <ol>
                        <li>{s(lang,
                            "Point at any other player and declare the attack. Attacker and Defender are now announced.",
                            "Zeige auf einen anderen Spieler und erkläre den Angriff. Angreifer und Verteidiger sind nun bekannt."
                        )}</li>
                        <li>{s(lang,
                            "Attacker places character card sword-side up; Defender places shield-side up.",
                            "Angreifer legt Personenkarte mit Schwert nach oben; Verteidiger mit Schild nach oben."
                        )}</li>
                        <li>{s(lang,
                            "Starting with the attacker's left neighbor, all other players declare support clockwise: support attacker (sword), support defender (shield), or abstain.",
                            "Ab dem linken Nachbarn des Angreifers deklarieren alle reihum: Angreifer (Schwert), Verteidiger (Schild) oder Enthaltung."
                        )}</li>
                        <li>{s(lang,
                            "Players may use occupation and object abilities in any order.",
                            "Spieler dürfen Berufs- und Gegenstandsfähigkeiten in beliebiger Reihenfolge einsetzen."
                        )}</li>
                        <li>{s(lang,
                            "Resolve: count face-up swords (attacker score) and shields (defender score). Cards showing both: owner chooses which counts. Higher total wins. Ties go to the defender.",
                            "Auflösen: Schwert-Symbole zählen für Angreifer, Schild-Symbole für Verteidiger. Karten mit beiden: Besitzer wählt. Höhere Summe gewinnt. Patt: Verteidiger gewinnt."
                        )}</li>
                        <li>{s(lang,
                            "Winner decides the loser's fate - either look at their occupation AND association cards, OR search luggage (loser shows all objects; winner takes one face-down).",
                            "Sieger entscheidet über den Verlierer: Entweder Berufs- UND Gesellschaftskarte ansehen, ODER Gepäck durchsuchen (Verlierer zeigt alle Gegenstände; Sieger nimmt einen verdeckt)."
                        )}</li>
                        <li>{s(lang,
                            "Tie: attacker draws one card from the draw pile if available.",
                            "Patt: Angreifer zieht eine Karte vom Stapel, falls vorhanden."
                        )}</li>
                    </ol>
                </div>

                <div class="rules-action-card">
                    <p class="rules-action-title">{"D. "}{s(lang, "Proclaim Victory", "Sieg verkünden")}</p>
                    <p>{s(lang,
                        "A player may proclaim victory if their association collectively holds the required objects AND they personally hold at least one.",
                        "Ein Spieler darf den Sieg verkünden, wenn seine Gesellschaft zusammen die nötigen Gegenstände besitzt UND er selbst mindestens einen davon hält."
                    )}</p>
                    <table class="table is-narrow is-bordered mt-2">
                        <thead><tr>
                            <th>{s(lang, "Association", "Gesellschaft")}</th>
                            <th>{s(lang, "Required", "Benötigt")}</th>
                        </tr></thead>
                        <tbody>
                            <tr>
                                <td>{s(lang, "Order of Open Secrets", "Orden der offenen Geheimnisse")}</td>
                                <td>{s(lang, "3 Keys", "3 Schlüssel")}</td>
                            </tr>
                            <tr>
                                <td>{s(lang, "Brotherhood of True Lies", "Bruderschaft der wahren Lügen")}</td>
                                <td>{s(lang, "3 Goblets", "3 Kelche")}</td>
                            </tr>
                        </tbody>
                    </table>
                    <p class="has-text-grey">{s(lang,
                        "Alternative: 2 Keys/Goblets + 1 matching Secret Bag. With an odd number of players, the minority association may also use a 'Drink of Power' card in place of one winning object.",
                        "Alternative: 2 Schlüssel/Kelche + 1 passender Geheimer Koffer. Bei ungerader Spielerzahl kann die Minderheitsgesellschaft einen Trank der Macht als Ersatz für einen Sieggegenstand nutzen."
                    )}</p>
                    <p class="mt-2">{s(lang,
                        "Procedure: stand up, make the declaration, reveal your association/occupation/objects, name your allies, they reveal their cards. Correct -> your association wins. Wrong -> the opposing association wins.",
                        "Ablauf: Aufstehen, Verkündigung aussprechen, eigene Karten und Gegenstände aufdecken, Verbündete nennen, diese decken ihre Karten auf. Korrekt -> deine Gesellschaft gewinnt. Falsch -> die Gegengesellschaft gewinnt."
                    )}</p>
                </div>
            </div>

            // ── Handlimit ─────────────────────────────────────────────────────
            <h2 class="title is-4 rules-section-title">{s(lang, "Hand Limit", "Handkartenlimit")}</h2>
            <p>{s(lang,
                "If a player ever holds too many objects, they must immediately give away the excess to any other player(s) of their choice. Objects may not be given away for any other reason.",
                "Hat ein Spieler zu viele Gegenstände, muss er die überschüssigen sofort an Mitspieler seiner Wahl abgeben. Gegenstände dürfen aus keinem anderen Grund abgegeben werden."
            )}</p>
            <table class="table is-narrow is-bordered">
                <thead><tr>
                    <th>{s(lang, "Players", "Spieler")}</th>
                    <th>{s(lang, "Max. objects", "Max. Gegenstände")}</th>
                </tr></thead>
                <tbody>
                    <tr><td>{"3"}</td><td>{"8"}</td></tr>
                    <tr><td>{"4"}</td><td>{"6"}</td></tr>
                    <tr><td>{"5+"}</td><td>{"5"}</td></tr>
                </tbody>
            </table>

            // ── Berufe ────────────────────────────────────────────────────────
            <h2 class="title is-4 rules-section-title">{s(lang, "Occupations", "Berufe")}</h2>
            <p class="mb-4">{s(lang,
                "Each player receives one secret occupation card at the start. Remaining cards form a face-down draw pile. When used, reveal the card and read it aloud; used cards remain face-up. Once-only occupations can be reactivated via the Coat or Tome objects.",
                "Jeder Spieler erhält zu Beginn eine geheime Berufskarte. Übrige Karten bilden einen verdeckten Stapel. Beim Einsetzen die Karte aufdecken und vorlesen; eingesetzte Karten bleiben offen. Einmalige Berufe können durch Mantel oder Foliant reaktiviert werden."
            )}</p>
            <div class="rules-cards-grid">
                { for ALL_JOBS.iter().map(|&job| html! {
                    <div class="card rules-card">
                        <div class="card-header">
                            <p class="card-header-title">{job.tr_name(lang)}</p>
                        </div>
                        <div class="card-content">
                            <p>{job.tr_desc(lang)}</p>
                        </div>
                    </div>
                })}
            </div>

            // ── Gegenstände ───────────────────────────────────────────────────
            <h2 class="title is-4 rules-section-title">{s(lang, "Objects", "Gegenstände")}</h2>
            <p class="mb-4">{s(lang,
                "Objects are held secretly (face-down). They can grant combat bonuses, reveal information, or trigger powerful effects when traded. Beginner tip: for the first game use only objects marked with a seal.",
                "Gegenstände werden verdeckt gehalten. Sie können Kampfboni gewähren, Informationen enthüllen oder beim Handel mächtige Effekte auslösen. Anfängertipp: Beim ersten Spiel nur Gegenstände mit Siegel verwenden."
            )}</p>
            <div class="rules-cards-grid">
                { for ALL_ITEMS.iter().map(|&item| html! {
                    <div class="card rules-card">
                        <div class="card-header">
                            <p class="card-header-title">{item_emoji(item)}{" "}{item.tr_name(lang)}</p>
                        </div>
                        <div class="card-content">
                            <p>{item.tr_desc(lang)}</p>
                        </div>
                    </div>
                })}
            </div>

            // ── Tipps ─────────────────────────────────────────────────────────
            <h2 class="title is-4 rules-section-title">{s(lang, "Tips for First-Time Players", "Tipps für die erste Runde")}</h2>
            <div class="notification is-info is-light">
                <ul>
                    <li>{s(lang,
                        "Don't try to memorize everything - it isn't necessary to win.",
                        "Versuche nicht, alles zu merken - es ist nicht notwendig, um zu gewinnen."
                    )}</li>
                    <li>{s(lang,
                        "Trade objects early even if they seem good; you'll get other useful ones in return.",
                        "Handle Gegenstände früh weiter, auch wenn sie gut erscheinen; du bekommst andere nützliche zurück."
                    )}</li>
                    <li>{s(lang,
                        "Don't refuse an object just because its effect benefits your opponent - next trade, you'll have that advantage.",
                        "Lehne einen Gegenstand nicht ab, nur weil sein Effekt dem Gegner nützt - beim nächsten Tausch hast du den Vorteil."
                    )}</li>
                    <li>{s(lang,
                        "Both Secret Bags are in the game from the start; trade them, don't hoard them.",
                        "Beide Geheimen Koffer sind von Anfang an im Spiel; handle sie weiter, horte sie nicht."
                    )}</li>
                    <li>{s(lang,
                        "If you know someone is on your side, signal it by supporting them in combat or trading them helpful objects.",
                        "Weißt du, wer auf deiner Seite ist, signalisiere es durch Unterstützung im Kampf oder hilfreiche Tauschgeschäfte."
                    )}</li>
                    <li>{s(lang,
                        "If you have an unwanted object, attack someone. Winning gives information; a tie lets you draw from the pile; even losing can reveal useful information.",
                        "Hast du einen unerwünschten Gegenstand, greife jemanden an. Gewinnen gibt Infos; Patt lässt dich vom Stapel ziehen; selbst Verlieren kann aufschlussreich sein."
                    )}</li>
                </ul>
            </div>

        </div>
    }
}
