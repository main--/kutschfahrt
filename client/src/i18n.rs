use web_protocol::{Item, Job, Faction, ActionLogEntry};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Lang { De, En }

impl Default for Lang {
    fn default() -> Self { Lang::De }
}

// ── Trait für Items und Jobs ──────────────────────────────────────────────────

pub trait Translate: Copy {
    fn tr_name(self, lang: Lang) -> &'static str;
    fn tr_desc(self, lang: Lang) -> &'static str;
    fn tr_emoji(self) -> &'static str;
    fn tr_tooltip(self, lang: Lang) -> String {
        format!("{}\n{}", self.tr_name(lang), self.tr_desc(lang))
    }
}

impl Translate for Item {
    fn tr_name(self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => match self {
                Item::Key              => "Key",
                Item::Goblet           => "Goblet",
                Item::BagKey           => "Secret Bag (Key)",
                Item::BagGoblet        => "Secret Bag (Goblet)",
                Item::BlackPearl       => "Black Pearl",
                Item::Dagger           => "Dagger",
                Item::Gloves           => "Gloves",
                Item::PoisonRing       => "Poison Ring",
                Item::CastingKnives    => "Casting Knives",
                Item::Whip             => "Whip",
                Item::Priviledge       => "Privilege",
                Item::Monocle          => "Monocle",
                Item::BrokenMirror     => "Broken Mirror",
                Item::Sextant          => "Sextant",
                Item::Coat             => "Coat",
                Item::Tome             => "Tome",
                Item::CoatOfArmorOfTheLoge => "Coat of Armor of the Loge",
            },
            Lang::De => match self {
                Item::Key              => "Schlüssel",
                Item::Goblet           => "Kelch",
                Item::BagKey           => "Geheimer Koffer (Schlüssel)",
                Item::BagGoblet        => "Geheimer Koffer (Kelch)",
                Item::BlackPearl       => "Schwarze Perle",
                Item::Dagger           => "Dolch",
                Item::Gloves           => "Handschuhe",
                Item::PoisonRing       => "Giftring",
                Item::CastingKnives    => "Wurfmesser",
                Item::Whip             => "Peitsche",
                Item::Priviledge       => "Freibrief",
                Item::Monocle          => "Monokel",
                Item::BrokenMirror     => "Zerbrochener Spiegel",
                Item::Sextant          => "Sextant",
                Item::Coat             => "Mantel",
                Item::Tome             => "Foliant",
                Item::CoatOfArmorOfTheLoge => "Wappen der Loge",
            },
        }
    }

    fn tr_desc(self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => match self {
                Item::Key =>
                    "The Order wins if they collectively hold at least 3 Keys.",
                Item::Goblet =>
                    "The Brotherhood wins if they collectively hold at least 3 Goblets.",
                Item::BagKey =>
                    "Trade it: draw one item from the pile. Cannot be traded for the other bag. Becomes a Key when the draw pile is empty.",
                Item::BagGoblet =>
                    "Trade it: draw one item from the pile. Cannot be traded for the other bag. Becomes a Goblet when the draw pile is empty.",
                Item::BlackPearl =>
                    "Must always be accepted in a trade. Holder cannot proclaim victory. (4–10 players only)",
                Item::Dagger =>
                    "As attacker, +1 to your result. Does not count when supporting.",
                Item::Gloves =>
                    "As defender, +1 to your result. Does not count when supporting.",
                Item::PoisonRing =>
                    "As attacker or defender, you win a tie.",
                Item::CastingKnives =>
                    "When you support an attacker, they get +1 to their result.",
                Item::Whip =>
                    "When you support a defender, they get +1 to their result.",
                Item::Priviledge =>
                    "Trade it: look at all of your trading partner's items.",
                Item::Monocle =>
                    "Trade it: look at your trading partner's faction card.",
                Item::BrokenMirror =>
                    "Must always be accepted in a trade. The item received in exchange has no trade effect.",
                Item::Sextant =>
                    "Trade it: choose a direction. All players simultaneously pass one item to their neighbor in that direction.",
                Item::Coat =>
                    "Trade it: pick a new job from the remaining pile. Return your old job. Once-only jobs are reset. (3–9 players only)",
                Item::Tome =>
                    "Trade it: swap jobs with your trading partner. Face-up jobs are turned face-down and may be used again.",
                Item::CoatOfArmorOfTheLoge =>
                    "If you hold this and 3 keys/goblets in any combination, proclaim sole victory — no allies needed. Drink of Power cards do not count.",
            },
            Lang::De => match self {
                Item::Key =>
                    "Der Orden kann den Sieg verkünden, wenn er mindestens 3 Schlüssel besitzt.",
                Item::Goblet =>
                    "Die Bruderschaft kann den Sieg verkünden, wenn sie mindestens 3 Kelche besitzt.",
                Item::BagKey =>
                    "Tauschst du ihn weiter, darfst du einen Gegenstand vom Stapel ziehen. Darf nicht gegen einen anderen Koffer getauscht werden. Gilt als Schlüssel, wenn der Stapel leer ist.",
                Item::BagGoblet =>
                    "Tauschst du ihn weiter, darfst du einen Gegenstand vom Stapel ziehen. Darf nicht gegen einen anderen Koffer getauscht werden. Gilt als Kelch, wenn der Stapel leer ist.",
                Item::BlackPearl =>
                    "Muss im Tausch angenommen werden. Wer sie besitzt, darf den Sieg nicht verkünden. (Nur für 4–10 Spieler)",
                Item::Dagger =>
                    "Als Angreifer erhältst du +1 auf dein Kampfergebnis. Gilt nicht beim Unterstützen.",
                Item::Gloves =>
                    "Als Verteidiger erhältst du +1 auf dein Kampfergebnis. Gilt nicht beim Unterstützen.",
                Item::PoisonRing =>
                    "Als Angreifer oder Verteidiger gewinnst du auch bei einem Patt.",
                Item::CastingKnives =>
                    "Unterstützt du einen Angreifer, erhält dieser +1 auf sein Kampfergebnis.",
                Item::Whip =>
                    "Unterstützt du einen Verteidiger, erhält dieser +1 auf sein Kampfergebnis.",
                Item::Priviledge =>
                    "Tauschst du ihn weiter, darfst du alle Gegenstände deines Tauschpartners ansehen.",
                Item::Monocle =>
                    "Tauschst du es weiter, darfst du die Gesellschaft deines Tauschpartners ansehen.",
                Item::BrokenMirror =>
                    "Muss im Tausch angenommen werden. Der erhaltene Gegenstand wirkt sich beim Tausch nicht aus.",
                Item::Sextant =>
                    "Tauschst du ihn weiter, bestimme eine Richtung. Alle geben gleichzeitig einen Gegenstand ihrer Wahl an den Nachbarn in dieser Richtung weiter.",
                Item::Coat =>
                    "Tauschst du ihn weiter, wähle einen neuen Beruf aus dem Stapel. Lege deinen alten Beruf zurück. Einmalige Berufe werden zurückgesetzt. (Nur für 3–9 Spieler)",
                Item::Tome =>
                    "Tauschst du ihn weiter, tauscht du mit deinem Tauschpartner die Berufe. Aufgedeckte Berufe werden wieder verdeckt und können erneut genutzt werden.",
                Item::CoatOfArmorOfTheLoge =>
                    "Besitzt du das Wappen und insgesamt 3 Kelche oder Schlüssel in beliebiger Kombination, darfst du deinen alleinigen Sieg verkünden. Karten 'Trank der Macht' zählen nicht.",
            },
        }
    }

    fn tr_emoji(self) -> &'static str {
        match self {
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
}

impl Translate for Job {
    fn tr_name(self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => match self {
                Job::Thug        => "Thug",
                Job::GrandMaster => "Grandmaster",
                Job::Bodyguard   => "Bodyguard",
                Job::Duelist     => "Duelist",
                Job::PoisonMixer => "Poison Mixer",
                Job::Doctor      => "Doctor",
                Job::Priest      => "Priest",
                Job::Hypnotist   => "Hypnotist",
                Job::Diplomat    => "Diplomat",
                Job::Clairvoyant => "Clairvoyant",
            },
            Lang::De => match self {
                Job::Thug        => "Schläger",
                Job::GrandMaster => "Großmeister",
                Job::Bodyguard   => "Leibwächter",
                Job::Duelist     => "Duellant",
                Job::PoisonMixer => "Giftmischer",
                Job::Doctor      => "Doktor",
                Job::Priest      => "Priester",
                Job::Hypnotist   => "Hypnotiseur",
                Job::Diplomat    => "Diplomat",
                Job::Clairvoyant => "Hellseher",
            },
        }
    }

    fn tr_desc(self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => match self {
                Job::Thug =>
                    "As the attacker in a struggle, you get +1 to your result. Does not count when defending or supporting.",
                Job::GrandMaster =>
                    "As the defender in a struggle, you get +1 to your result. May be used even after support has been declared.",
                Job::Bodyguard =>
                    "When you support someone in a struggle, they get +1 to their result. Does not apply to yourself.",
                Job::Duelist =>
                    "Once: As attacker or defender, declare that nobody may support this struggle. You get +1 to your result.",
                Job::PoisonMixer =>
                    "Once: Appoint the winner of a struggle. Cannot be used if you are the attacker or defender.",
                Job::Doctor =>
                    "Once: Immediately after a struggle, prevent all its effects. The winner gains no information and may not search the loser's items.",
                Job::Priest =>
                    "Once: Before support is declared, stop an attack. If the attacker owns at least 2 items, they must give you one of their choice.",
                Job::Hypnotist =>
                    "As attacker: after support is declared, force one player to abstain. That player may not use items or jobs this struggle.",
                Job::Diplomat =>
                    "Once, on your turn: demand a specific item from any player. If they deny having it, they must show you all their items.",
                Job::Clairvoyant =>
                    "Once: Look at the draw pile, pick 2 items, reshuffle, then place them on top in any order.",
            },
            Lang::De => match self {
                Job::Thug =>
                    "Als Angreifer erhältst du +1 auf dein Kampfergebnis. Gilt nicht beim Verteidigen oder Unterstützen.",
                Job::GrandMaster =>
                    "Als Verteidiger erhältst du +1 auf dein Kampfergebnis. Kann auch nach der Unterstützungsankündigung eingesetzt werden.",
                Job::Bodyguard =>
                    "Unterstützt du jemanden im Kampf, erhält dieser +1 auf sein Kampfergebnis. Gilt nicht für dich selbst.",
                Job::Duelist =>
                    "Einmalig: Als Angreifer oder Verteidiger kannst du bestimmen, dass niemand unterstützen darf. Du erhältst +1 auf dein Kampfergebnis.",
                Job::PoisonMixer =>
                    "Einmalig: Bestimme den Sieger eines Kampfes. Gilt nicht, wenn du Angreifer oder Verteidiger bist.",
                Job::Doctor =>
                    "Einmalig: Verhindere direkt nach einem Kampf dessen Auswirkungen. Der Sieger erfährt nichts und darf nicht ins Gepäck schauen.",
                Job::Priest =>
                    "Einmalig: Verhindere vor der Unterstützungsankündigung einen Kampf. Hat der Angreifer mindestens 2 Gegenstände, muss er dir einen abgeben.",
                Job::Hypnotist =>
                    "Als Angreifer: Direkt nach der Unterstützungsankündigung kannst du einen Spieler zur Enthaltung zwingen. Er darf keine Gegenstände oder Berufe einsetzen.",
                Job::Diplomat =>
                    "Einmalig in deinem Zug: Verlange einen bestimmten Gegenstand von einem Mitspieler. Behauptet er, ihn nicht zu haben, muss er alle Gegenstände zeigen.",
                Job::Clairvoyant =>
                    "Einmalig: Schaue den Stapel an, wähle 2 Gegenstände, mische den Stapel und lege die gewählten Gegenstände in beliebiger Reihenfolge oben auf.",
            },
        }
    }

    fn tr_emoji(self) -> &'static str {
        match self {
            Job::Thug        => "💪",
            Job::GrandMaster => "⚜️",
            Job::Bodyguard   => "💂",
            Job::Duelist     => "🤺",
            Job::PoisonMixer => "⚗️",
            Job::Doctor      => "🩺",
            Job::Priest      => "🙏",
            Job::Hypnotist   => "🌀",
            Job::Diplomat    => "🎭",
            Job::Clairvoyant => "🔮",
        }
    }
}

// ── Gesellschaften ────────────────────────────────────────────────────────────

pub fn faction_name(faction: Faction, lang: Lang) -> &'static str {
    match lang {
        Lang::En => match faction {
            Faction::Order        => "Order of Open Secrets",
            Faction::Brotherhood  => "Brotherhood of True Lies",
        },
        Lang::De => match faction {
            Faction::Order        => "Orden der offenen Geheimnisse",
            Faction::Brotherhood  => "Bruderschaft der wahren Lüge",
        },
    }
}

pub fn faction_victory_tip(faction: Faction, lang: Lang) -> &'static str {
    match lang {
        Lang::En => match faction {
            Faction::Order       => "🔑 Victory: all Order members together hold at least 3 Keys (Secret Bags count once the draw pile is empty).",
            Faction::Brotherhood => "🏆 Victory: all Brotherhood members together hold at least 3 Goblets (Secret Bags count once the draw pile is empty).",
        },
        Lang::De => match faction {
            Faction::Order       => "🔑 Sieg: Alle Ordensmitglieder besitzen zusammen mindestens 3 Schlüssel (Geheime Koffer zählen, wenn der Stapel leer ist).",
            Faction::Brotherhood => "🏆 Sieg: Alle Bruderschaftsmitglieder besitzen zusammen mindestens 3 Kelche (Geheime Koffer zählen, wenn der Stapel leer ist).",
        },
    }
}

// ── Action-Log Strings ────────────────────────────────────────────────────────

pub fn action_log_text(entry: &ActionLogEntry, lang: Lang) -> String {
    match *entry {
        ActionLogEntry::Pass { actor } => match lang {
            Lang::En => format!("{actor} passed."),
            Lang::De => format!("{actor} passt."),
        },
        ActionLogEntry::AnnounceVictory { actor } => match lang {
            Lang::En => format!("{actor} announced victory."),
            Lang::De => format!("{actor} verkündet den Sieg."),
        },
        ActionLogEntry::UseDiplomat { actor, target, item, success: true } => {
            let iname = item.tr_name(lang);
            match lang {
                Lang::En => format!("{actor} asked {target} for a {iname}. They exchanged items."),
                Lang::De => format!("{actor} verlangt von {target} einen {iname}. Sie tauschten Gegenstände."),
            }
        },
        ActionLogEntry::UseDiplomat { actor, target, item, success: false } => {
            let iname = item.tr_name(lang);
            match lang {
                Lang::En => format!("{actor} asked {target} for a {iname}, but {target} did not have one."),
                Lang::De => format!("{actor} verlangt von {target} einen {iname}, aber {target} hat ihn nicht."),
            }
        },
        ActionLogEntry::UseClairvoyant { actor } => match lang {
            Lang::En => format!("{actor} reordered the item stack."),
            Lang::De => format!("{actor} sortiert den Gegenstandsstapel um."),
        },
        ActionLogEntry::TradeOffer { offerer, target, accepted } => match lang {
            Lang::En => format!("{offerer} offered a trade to {target}. The trade was {}.",
                if accepted { "accepted" } else { "declined" }),
            Lang::De => format!("{offerer} bietet {target} einen Tausch an. Der Tausch wurde {}.",
                if accepted { "angenommen" } else { "abgelehnt" }),
        },
        ActionLogEntry::Attack { attacker, target } => match lang {
            Lang::En => format!("{attacker} attacked {target}."),
            Lang::De => format!("{attacker} greift {target} an."),
        },
        ActionLogEntry::TradeTrigger { giver, receiver, .. } => match lang {
            Lang::En => format!("{giver} passed an item to {receiver}."),
            Lang::De => format!("{giver} gibt {receiver} einen Gegenstand weiter."),
        },
        ActionLogEntry::DonateItem { giver, receiver } => match lang {
            Lang::En => format!("{giver} donates an item to {receiver}."),
            Lang::De => format!("{giver} schenkt {receiver} einen Gegenstand."),
        },
    }
}

// ── UI-Strings ────────────────────────────────────────────────────────────────

impl Lang {
    // Navbar / Home
    pub fn login(self)        -> &'static str { self.s("🔑 Login",    "🔑 Anmelden") }
    pub fn logout(self)       -> &'static str { self.s("🚪 Logout",   "🚪 Abmelden") }
    pub fn your_games(self)   -> &'static str { self.s("Your Games",  "Deine Spiele") }
    pub fn please_login(self) -> &'static str { self.s("Please log in.", "Bitte anmelden.") }
    pub fn game_label(self)   -> &'static str { self.s("Game",        "Spiel") }

    // Pregame
    pub fn invite_others(self)  -> &'static str { self.s("Invite others:", "Andere einladen:") }
    pub fn copy_link(self)      -> &'static str { self.s("📋 Copy link",   "📋 Link kopieren") }
    pub fn players_label(self)  -> &'static str { self.s("Players:",       "Spieler:") }
    pub fn join(self)           -> &'static str { self.s("🎭 Join",         "🎭 Beitreten") }
    pub fn leave(self)          -> &'static str { self.s("🚪 Leave",        "🚪 Verlassen") }
    pub fn start_game(self)     -> &'static str { self.s("🚀 Start Game",   "🚀 Spiel starten") }

    // HUD Labels
    pub fn player_col(self)     -> &'static str { self.s("Player",       "Spieler") }
    pub fn job_col(self)        -> &'static str { self.s("Job",          "Beruf") }
    pub fn items_col(self)      -> &'static str { self.s("Items",        "Gegenstände") }
    pub fn draw_pile(self)      -> &'static str { self.s("Draw pile",    "Nachziehstapel") }
    pub fn turn_indicator(self) -> &'static str { self.s("(turn)",       "(am Zug)") }
    pub fn your_job(self)       -> &'static str { self.s("Your job:",    "Dein Beruf:") }
    pub fn revealed(self)       -> &'static str { self.s("revealed",     "aufgedeckt") }
    pub fn not_revealed(self)   -> &'static str { self.s("not revealed", "verdeckt") }
    pub fn your_faction(self)   -> &'static str { self.s("Your faction:", "Deine Gesellschaft:") }
    pub fn your_faction_cards(self) -> &'static str { self.s("Your faction cards:", "Deine Gesellschaftskarten:") }
    pub fn your_items(self)     -> &'static str { self.s("Your items",   "Deine Gegenstände") }
    pub fn too_many_items(self, n: usize) -> String {
        match self {
            Lang::En => format!(" ⚠ Too many items! Donate {}.", n),
            Lang::De => format!(" ⚠ Zu viele Gegenstände! Verschenke {}.", n),
        }
    }

    // Turn actions
    pub fn attack(self)           -> &'static str { self.s("⚔️ Attack",                     "⚔️ Angreifen") }
    pub fn offer_trade(self)      -> &'static str { self.s("🤝 Offer Trade",                "🤝 Tausch anbieten") }
    pub fn announce_victory(self) -> &'static str { self.s("👑 Announce Victory",           "👑 Sieg verkünden") }
    pub fn pass(self)             -> &'static str { self.s("⏭ Pass",                        "⏭ Passen") }
    pub fn game_start(self)       -> &'static str { self.s("The game begins.",               "Das Spiel beginnt.") }
    pub fn black_pearl_no_victory(self)    -> &'static str { self.s("Cannot proclaim victory while holding the Black Pearl.", "Mit der Schwarzen Perle kann der Sieg nicht verkündet werden.") }
    pub fn black_pearl_must_accept(self)   -> &'static str { self.s("The Black Pearl must always be accepted.", "Die Schwarze Perle muss immer angenommen werden.") }
    pub fn broken_mirror_must_accept(self) -> &'static str { self.s("The Broken Mirror must always be accepted.", "Der Zerbrochene Spiegel muss immer angenommen werden.") }
    pub fn use_clairvoyant(self)  -> &'static str { self.s("🔮 Use Clairvoyant",            "🔮 Hellseher einsetzen") }
    pub fn use_diplomat(self)     -> &'static str { self.s("🎭 Use Diplomat",               "🎭 Diplomat einsetzen") }
    pub fn loge_victory(self)     -> &'static str { self.s("🌟 Announce Sole Victory (Loge)", "🌟 Alleinigen Sieg verkünden (Loge)") }
    pub fn end_turn(self)         -> &'static str { self.s("✅ End Turn",                   "✅ Zug beenden") }
    pub fn submit(self)           -> &'static str { self.s("✔ Submit",                      "✔ Bestätigen") }
    pub fn done(self)             -> &'static str { self.s("✔ Done",                        "✔ Fertig") }

    // Voting for victory
    pub fn announce_with(self) -> &'static str {
        self.s("Select your allies who hold the winning items:",
               "Wähle deine Verbündeten, die die Sieggegenstände besitzen:")
    }
    pub fn announce_alone(self) -> &'static str {
        self.s("(Announce alone if you hold all required items yourself.)",
               "(Alleine verkünden, wenn du alle nötigen Gegenstände selbst besitzt.)")
    }

    // Attack phase
    pub fn use_priest(self)            -> &'static str { self.s("✋ Use Priest",             "✋ Priester einsetzen") }
    pub fn dont(self)                  -> &'static str { self.s("❌ Don't",                  "❌ Nein") }
    pub fn support_attacker(self)      -> &'static str { self.s("⚔️ Support Attacker",       "⚔️ Angreifer unterstützen") }
    pub fn support_defender(self)      -> &'static str { self.s("🛡 Support Defender",       "🛡 Verteidiger unterstützen") }
    pub fn abstain(self)               -> &'static str { self.s("🤷 Abstain",                "🤷 Enthalten") }
    pub fn dont_hypnotize(self)        -> &'static str { self.s("❌ Don't hypnotize",         "❌ Nicht hypnotisieren") }
    pub fn hypnotize(self)             -> &'static str { self.s("🌀 Hypnotize",              "🌀 Hypnotisieren") }
    pub fn truth_reward(self)          -> &'static str { self.s("🔍 Truth (Faction & Job)", "🔍 Wahrheit (Gesellschaft & Beruf)") }
    pub fn items_reward(self)          -> &'static str { self.s("🎴 Items",                  "🎴 Gegenstände") }
    pub fn use_item(self)              -> &'static str { self.s("✨ Use item",               "✨ Gegenstand einsetzen") }
    pub fn make_attacker_win(self)     -> &'static str { self.s("⚔️ Make attacker win",      "⚔️ Angreifer gewinnen lassen") }
    pub fn make_defender_win(self)     -> &'static str { self.s("🛡 Make defender win",      "🛡 Verteidiger gewinnen lassen") }
    pub fn use_job(self, name: &str) -> String {
        match self {
            Lang::En => format!("🎭 Use job ({})", name),
            Lang::De => format!("🎭 Beruf einsetzen ({})", name),
        }
    }
    pub fn pass_items(self) -> &'static str { self.s("⏭ Pass", "⏭ Passen") }
    pub fn you_passed(self) -> &'static str { self.s("You have passed.", "Du hast gepasst.") }
    pub fn card(self, n: usize) -> String {
        match self {
            Lang::En => format!("{}️⃣ Card {}", n, n),
            Lang::De => format!("{}️⃣ Karte {}", n, n),
        }
    }
    pub fn declaring_support(self)   -> &'static str { self.s("Declaring support",   "Unterstützung ankündigen") }
    pub fn hypnotizing(self)         -> &'static str { self.s("Hypnotizing",          "Hypnotisieren") }
    pub fn items_and_jobs(self)      -> &'static str { self.s("Items & Jobs",         "Gegenstände & Berufe") }
    pub fn is_attacker(self, p: &str) -> String {
        match self {
            Lang::En => format!("{} is the attacker", p),
            Lang::De => format!("{} ist der Angreifer", p),
        }
    }
    pub fn is_defender(self, p: &str) -> String {
        match self {
            Lang::En => format!("{} is the defender", p),
            Lang::De => format!("{} ist der Verteidiger", p),
        }
    }
    pub fn active_buffs(self) -> &'static str { self.s("Active buffs:", "Aktive Boni:") }
    pub fn tally(self, a: &str, d: &str) -> String {
        match self {
            Lang::En => format!("Current tally: {} vs {}", a, d),
            Lang::De => format!("Aktueller Stand: {} zu {}", a, d),
        }
    }
    pub fn use_priest_prompt(self) -> &'static str {
        self.s("Use priest?", "Priester einsetzen?")
    }
    pub fn waiting_for_priest(self) -> &'static str {
        self.s("Waiting for other players to use Priest ...", "Warten auf andere Spieler (Priester) ...")
    }
    pub fn select_item_for_priest(self, p: &str) -> String {
        match self {
            Lang::En => format!("Select an item to give to the priest ({})", p),
            Lang::De => format!("Wähle einen Gegenstand für den Priester ({})", p),
        }
    }
    pub fn paying_priest_wait(self, attacker: &str, priest: &str) -> String {
        match self {
            Lang::En => format!("Waiting for {} to give an item to the Priest ({}) ...", attacker, priest),
            Lang::De => format!("Warten auf {}, einen Gegenstand an den Priester ({}) zu geben ...", attacker, priest),
        }
    }
    pub fn see_faction_job(self, opponent: &str, faction: &str, job: &str) -> String {
        match self {
            Lang::En => format!("You see that {}'s faction is {} and their job is {}.", opponent, faction, job),
            Lang::De => format!("Du siehst: {}'s Gesellschaft ist {} und sein Beruf ist {}.", opponent, faction, job),
        }
    }
    pub fn pick_faction_card(self, p: &str) -> String {
        match self {
            Lang::En => format!("Pick one of {}'s faction cards to look at:", p),
            Lang::De => format!("Wähle eine von {}'s Gesellschaftskarten:", p),
        }
    }
    pub fn steal_item_from(self, victim: &str) -> String {
        match self {
            Lang::En => format!("Select an item to steal from {}.", victim),
            Lang::De => format!("Wähle einen Gegenstand, den du von {} stiehlst.", victim),
        }
    }
    pub fn give_back_to(self, victim: &str) -> String {
        match self {
            Lang::En => format!("Select an item to give back to {}.", victim),
            Lang::De => format!("Wähle einen Gegenstand zum Zurückgeben an {}.", victim),
        }
    }
    pub fn waiting_for_reward(self, winner: &str) -> String {
        match self {
            Lang::En => format!("Waiting for {} to claim a reward ...", winner),
            Lang::De => format!("Warten auf {}, eine Belohnung zu wählen ...", winner),
        }
    }
    pub fn waiting_for_steal(self, winner: &str) -> String {
        match self {
            Lang::En => format!("Waiting for {} to steal items ...", winner),
            Lang::De => format!("Warten auf {}, Gegenstände zu stehlen ...", winner),
        }
    }
    pub fn waiting_for_faction_look(self, winner: &str) -> String {
        match self {
            Lang::En => format!("Waiting for {} to look at faction & job ...", winner),
            Lang::De => format!("Warten auf {}, Gesellschaft & Beruf anzusehen ...", winner),
        }
    }
    pub fn waiting_for_faction_look_n(self, winner: &str, n: usize) -> String {
        match self {
            Lang::En => format!("Waiting for {} to look at faction {} & job ...", winner, n),
            Lang::De => format!("Warten auf {}, Karte {} & Beruf anzusehen ...", winner, n),
        }
    }
    pub fn is_attacking(self, a: &str, d: &str) -> String {
        match self {
            Lang::En => format!("{} is attacking {}", a, d),
            Lang::De => format!("{} greift {} an", a, d),
        }
    }
    pub fn waiting_for_hypnotizer(self) -> &'static str {
        self.s("Waiting for hypnotizer ...", "Warten auf Hypnotiseur ...")
    }

    // Trading
    pub fn offering(self, offerer: &str, item: &str) -> String {
        match self {
            Lang::En => format!("{} is offering you a {}", offerer, item),
            Lang::De => format!("{} bietet dir einen {} an", offerer, item),
        }
    }
    /// Text before the item name in the trade offer sentence
    pub fn offering_before(self, offerer: &str) -> String {
        match self {
            Lang::En => format!("{} is offering you a ", offerer),
            Lang::De => format!("{} bietet dir einen ", offerer),
        }
    }
    /// Text after the item name (empty in EN, " an" in DE)
    pub fn offering_after(self) -> &'static str {
        self.s("", " an")
    }
    pub fn accept(self)  -> &'static str { self.s("✅ Accept",  "✅ Annehmen") }
    pub fn decline(self) -> &'static str { self.s("❌ Decline", "❌ Ablehnen") }
    pub fn select_item_hint(self) -> &'static str {
        self.s("Select an item from your inventory to give back:",
               "Wähle einen Gegenstand aus deinem Gepäck zum Zurückgeben:")
    }

    // Trade trigger
    pub fn inspecting_items(self, giver: &str, receiver: &str) -> String {
        match self {
            Lang::En => format!("Waiting for {} to inspect {}'s items ...", giver, receiver),
            Lang::De => format!("Warten auf {}, um {}'s Gegenstände anzusehen ...", giver, receiver),
        }
    }
    pub fn items_of(self, receiver: &str, items: &str) -> String {
        match self {
            Lang::En => format!("{}'s items: {}", receiver, items),
            Lang::De => format!("{}'s Gegenstände: {}", receiver, items),
        }
    }
    pub fn looking_at_faction(self, giver: &str, receiver: &str) -> String {
        match self {
            Lang::En => format!("Waiting for {} to look at {}'s faction ...", giver, receiver),
            Lang::De => format!("Warten auf {}, um {}'s Gesellschaft anzusehen ...", giver, receiver),
        }
    }
    pub fn faction_of(self, receiver: &str, idx: Option<usize>, faction: &str) -> String {
        let card = idx.map(|i| match self {
            Lang::En => format!(" (card {})", i + 1),
            Lang::De => format!(" (Karte {})", i + 1),
        }).unwrap_or_default();
        match self {
            Lang::En => format!("{}'s faction{}: {}", receiver, card, faction),
            Lang::De => format!("{}'s Gesellschaft{}: {}", receiver, card, faction),
        }
    }
    pub fn waiting_for_new_job(self, giver: &str) -> String {
        match self {
            Lang::En => format!("Waiting for {} to pick a new job ...", giver),
            Lang::De => format!("Warten auf {}, einen neuen Beruf zu wählen ...", giver),
        }
    }
    pub fn pick_new_job(self)    -> &'static str { self.s("Pick your new job:",             "Wähle deinen neuen Beruf:") }
    pub fn sextant_intro(self)   -> &'static str { self.s("Sextant: each player selects an item to pass around.", "Sextant: Jeder Spieler wählt einen Gegenstand zum Weitergeben.") }
    pub fn sextant_passes(self, p: &str, i: &str) -> String {
        match self {
            Lang::En => format!("{} passes: {}", p, i),
            Lang::De => format!("{} gibt weiter: {}", p, i),
        }
    }
    pub fn sextant_select(self)  -> &'static str { self.s("Select an item from your inventory to pass:", "Wähle einen Gegenstand aus deinem Gepäck:") }
    pub fn pass_item(self)       -> &'static str { self.s("➡️ Pass this item",  "➡️ Diesen Gegenstand weitergeben") }
    pub fn choose_direction(self)-> &'static str { self.s("Choose direction to pass items:", "Wähle die Weitergabe-Richtung:") }
    pub fn forward(self)         -> &'static str { self.s("➡️ Forward",   "➡️ Vorwärts") }
    pub fn backward(self)        -> &'static str { self.s("⬅️ Backward",  "⬅️ Rückwärts") }
    pub fn waiting_others(self)  -> &'static str { self.s("Waiting for other players ...", "Warten auf andere Spieler ...") }

    // Clairvoyant
    pub fn item_stack_label(self) -> &'static str { self.s("Item stack:", "Gegenstandsstapel:") }

    // Victory
    pub fn victory_order(self)        -> &'static str {
        self.s("The Order of Open Secrets is victorious!",
               "Der Orden der offenen Geheimnisse siegt!")
    }
    pub fn victory_brotherhood(self)  -> &'static str {
        self.s("The Brotherhood of True Lies is victorious!",
               "Die Bruderschaft der wahren Lüge siegt!")
    }
    pub fn victory_traitor(self, p: &str) -> String {
        match self {
            Lang::En => format!("{} wins alone! (Loge)", p),
            Lang::De => format!("{} siegt alleine! (Loge)", p),
        }
    }
    pub fn victory_confirm(self)      -> &'static str {
        self.s("Really announce victory? This cannot be undone.",
               "Sieg wirklich verkünden? Dies kann nicht rückgängig gemacht werden.")
    }
    pub fn confirm_yes(self)          -> &'static str { self.s("✔ Yes, announce!",      "✔ Ja, verkünden!") }
    pub fn confirm_no(self)           -> &'static str { self.s("✖ Cancel",              "✖ Abbrechen") }

    // Waiting states
    pub fn waiting_for(self, p: &str) -> String {
        match self {
            Lang::En => format!("Waiting for {} ...", p),
            Lang::De => format!("Warten auf {} ...", p),
        }
    }
    pub fn waiting_for_trade(self, offerer: &str, target: &str) -> String {
        match self {
            Lang::En => format!("{} is offering an item to {} ...", offerer, target),
            Lang::De => format!("{} bietet {} einen Gegenstand an ...", offerer, target),
        }
    }
    pub fn waiting_for_donate(self, donor: &str) -> String {
        match self {
            Lang::En => format!("Waiting for {} to donate an item ...", donor),
            Lang::De => format!("Warten auf {}, einen Gegenstand zu verschenken ...", donor),
        }
    }
    pub fn waiting_for_clairvoyant(self, p: &str) -> String {
        match self {
            Lang::En => format!("Waiting for the Clairvoyant ({}) to do their work ...", p),
            Lang::De => format!("Warten auf den Hellseher ({}) ...", p),
        }
    }
    pub fn waiting_for_diplomat(self, diplomat: &str, target: &str) -> String {
        match self {
            Lang::En => format!("Waiting for the Diplomat ({}) to confirm that {} does not have the requested item ...", diplomat, target),
            Lang::De => format!("Warten auf den Diplomaten ({}), um zu bestätigen, dass {} den Gegenstand nicht hat ...", diplomat, target),
        }
    }
    pub fn diplomat_no_item(self, target: &str, items: &str) -> String {
        match self {
            Lang::En => format!("Since {} does not have the requested item, you may see their inventory: {}", target, items),
            Lang::De => format!("Da {} den Gegenstand nicht hat, siehst du sein Gepäck: {}", target, items),
        }
    }
    pub fn ask_for(self)         -> &'static str { self.s("Ask for:", "Verlange:") }

    // Spectator
    pub fn spectating(self)      -> &'static str { self.s("👁 Spectating",   "👁 Zuschauer") }
    pub fn spec_turn(self, p: &str) -> String {
        match self {
            Lang::En => format!("{}'s turn", p),
            Lang::De => format!("{} ist am Zug", p),
        }
    }
    pub fn spec_ending_turn(self, p: &str) -> String {
        match self {
            Lang::En => format!("{} is ending their turn", p),
            Lang::De => format!("{} beendet seinen Zug", p),
        }
    }
    pub fn spec_donating(self, p: &str) -> String {
        match self {
            Lang::En => format!("{} is donating an item", p),
            Lang::De => format!("{} verschenkt einen Gegenstand", p),
        }
    }
    pub fn spec_trade(self, a: &str, b: &str) -> String {
        match self {
            Lang::En => format!("{} is offering an item to {}", a, b),
            Lang::De => format!("{} bietet {} einen Gegenstand an", a, b),
        }
    }
    pub fn spec_trigger(self, a: &str, b: &str) -> String {
        match self {
            Lang::En => format!("Resolving trade effect between {} and {}", a, b),
            Lang::De => format!("Tauscheffekt zwischen {} und {} wird aufgelöst", a, b),
        }
    }
    pub fn spec_clairvoyant(self, p: &str) -> String {
        match self {
            Lang::En => format!("{} (Clairvoyant) is reordering the draw pile", p),
            Lang::De => format!("{} (Hellseher) sortiert den Stapel um", p),
        }
    }
    pub fn spec_diplomat(self, d: &str, t: &str) -> String {
        match self {
            Lang::En => format!("{} (Diplomat) is reviewing {}'s inventory", d, t),
            Lang::De => format!("{} (Diplomat) prüft {}'s Gepäck", d, t),
        }
    }
    pub fn spec_gameover_order(self)    -> &'static str {
        self.s("Game over — the Order wins!", "Spiel vorbei — der Orden siegt!")
    }
    pub fn spec_gameover_brotherhood(self) -> &'static str {
        self.s("Game over — the Brotherhood wins!", "Spiel vorbei — die Bruderschaft siegt!")
    }
    pub fn spec_gameover_traitor(self, p: &str) -> String {
        match self {
            Lang::En => format!("Game over — {} wins alone!", p),
            Lang::De => format!("Spiel vorbei — {} siegt alleine!", p),
        }
    }

    // Attack phase spectator subtitles
    pub fn spec_attack_waiting_priest(self) -> &'static str { self.s("waiting for the Priest", "Priester wird abgewartet") }
    pub fn spec_attack_priest_stopped(self) -> &'static str { self.s("Priest stopped the attack", "Priester stoppt den Angriff") }
    pub fn spec_attack_support(self)        -> &'static str { self.s("declaring support", "Unterstützung ankündigen") }
    pub fn spec_attack_hypnotist(self)      -> &'static str { self.s("Hypnotist is choosing a target", "Hypnotiseur wählt ein Ziel") }
    pub fn spec_attack_items(self)          -> &'static str { self.s("playing items & jobs", "Gegenstände & Berufe einsetzen") }
    pub fn spec_attack_resolving(self)      -> &'static str { self.s("resolving", "Auflösung") }
    pub fn spec_attack_reward(self)         -> &'static str { self.s("winner choosing reward", "Sieger wählt Belohnung") }

    // Donation
    pub fn donate_prompt(self) -> &'static str {
        self.s("Select a player and an item to donate:",
               "Wähle einen Spieler und einen Gegenstand zum Verschenken:")
    }

    // Misc / misc labels
    pub fn you_label(self)             -> &'static str { self.s("(you)", "(du)") }
    pub fn pick_any_faction_card(self) -> &'static str {
        self.s("Pick a faction card to look at:", "Wähle eine Gesellschaftskarte:")
    }
    pub fn use_item_prompt(self)       -> &'static str { self.s("Use an item:", "Gegenstand einsetzen:") }
    pub fn alone_word(self)            -> &'static str { self.s("alone", "alleine") }
    pub fn together_with_word(self)    -> &'static str { self.s("together with", "zusammen mit") }
    pub fn and_word(self)              -> &'static str { self.s("and", "und") }

    // TurnStart action preview texts
    pub fn will_pass(self) -> &'static str {
        self.s("You are going to pass.", "Du wirst passen.")
    }
    pub fn will_loge_victory(self) -> &'static str {
        self.s("You are going to use the Coat of Arms of the Loge to win alone.",
               "Du wirst das Wappen der Loge einsetzen, um alleine zu siegen.")
    }
    pub fn will_announce_victory(self, faction: &str, allies: &str) -> String {
        match self {
            Lang::En => format!("You are going to announce the victory of the {} {}.", faction, allies),
            Lang::De => format!("Du wirst den Sieg der {} {} verkünden.", faction, allies),
        }
    }
    pub fn will_offer_trade(self, item: &str, target: &str) -> String {
        match self {
            Lang::En => format!("You offer to trade a {} to {}.", item, target),
            Lang::De => format!("Du bietest {} einen {} an.", target, item),
        }
    }
    pub fn will_attack(self, target: &str) -> String {
        match self {
            Lang::En => format!("You attack {}.", target),
            Lang::De => format!("Du greifst {} an.", target),
        }
    }
    pub fn will_clairvoyant(self) -> &'static str {
        self.s("You are going to use your job ability (Clairvoyant).",
               "Du wirst deine Berufsfähigkeit (Hellseher) einsetzen.")
    }
    pub fn will_diplomat(self, demand: &str, target: &str, give: &str) -> String {
        match self {
            Lang::En => format!("You demand a {} from {}. You will give them a {}.",
                demand, target, give),
            Lang::De => format!("Du verlangst einen {} von {}. Du gibst einen {} ab.",
                demand, target, give),
        }
    }

    // ── helper ────────────────────────────────────────────────────────────────
    fn s(self, en: &'static str, de: &'static str) -> &'static str {
        match self { Lang::En => en, Lang::De => de }
    }
}

