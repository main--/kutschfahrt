use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;
use web_protocol::{Item, Job};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Lang { #[default] De, En }

// ── Fluent bundles ────────────────────────────────────────────────────────────

thread_local! {
    static DE_BUNDLE: FluentBundle<FluentResource> =
        init_bundle("de", include_str!("locales/de.ftl"));
    static EN_BUNDLE: FluentBundle<FluentResource> =
        init_bundle("en", include_str!("locales/en.ftl"));
}

fn init_bundle(lang: &str, source: &'static str) -> FluentBundle<FluentResource> {
    let langid: LanguageIdentifier = lang.parse().expect("invalid language identifier");
    let res = FluentResource::try_new(source.to_owned()).expect("FTL parse error");
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.add_resource(res).expect("failed to add FTL resource");
    bundle
}

fn ftl_msg(bundle: &FluentBundle<FluentResource>, key: &str) -> String {
    let msg = bundle
        .get_message(key)
        .unwrap_or_else(|| panic!("missing FTL message: {key}"));
    let val = msg
        .value()
        .unwrap_or_else(|| panic!("FTL message has no value: {key}"));
    let mut errors = vec![];
    bundle.format_pattern(val, None, &mut errors).into_owned()
}

fn lookup(lang: Lang, key: &str) -> String {
    match lang {
        Lang::De => DE_BUNDLE.with(|b| ftl_msg(b, key)),
        Lang::En => EN_BUNDLE.with(|b| ftl_msg(b, key)),
    }
}

/// Emojis are language-independent; always resolved from the DE bundle.
fn lookup_emoji(key: &str) -> String {
    DE_BUNDLE.with(|b| ftl_msg(b, key))
}

// ── Translate trait ───────────────────────────────────────────────────────────

pub trait Translate: Copy {
    fn tr_name(self, lang: Lang) -> String;
    fn tr_desc(self, lang: Lang) -> String;
    fn tr_emoji(self) -> String;
    fn tr_tooltip(self, lang: Lang) -> String {
        format!("{}\n{}", self.tr_name(lang), self.tr_desc(lang))
    }
}

// ── Item ─────────────────────────────────────────────────────────────────────

impl Translate for Item {
    fn tr_name(self, lang: Lang) -> String {
        lookup(lang, match self {
            Item::Key                  => "item-key-name",
            Item::Goblet               => "item-goblet-name",
            Item::BagKey               => "item-bag-key-name",
            Item::BagGoblet            => "item-bag-goblet-name",
            Item::BlackPearl           => "item-black-pearl-name",
            Item::Dagger               => "item-dagger-name",
            Item::Gloves               => "item-gloves-name",
            Item::PoisonRing           => "item-poison-ring-name",
            Item::CastingKnives        => "item-casting-knives-name",
            Item::Whip                 => "item-whip-name",
            Item::Priviledge           => "item-priviledge-name",
            Item::Monocle              => "item-monocle-name",
            Item::BrokenMirror         => "item-broken-mirror-name",
            Item::Sextant              => "item-sextant-name",
            Item::Coat                 => "item-coat-name",
            Item::Tome                 => "item-tome-name",
            Item::CoatOfArmorOfTheLoge => "item-coat-of-armor-name",
        })
    }

    fn tr_desc(self, lang: Lang) -> String {
        lookup(lang, match self {
            Item::Key                  => "item-key-desc",
            Item::Goblet               => "item-goblet-desc",
            Item::BagKey               => "item-bag-key-desc",
            Item::BagGoblet            => "item-bag-goblet-desc",
            Item::BlackPearl           => "item-black-pearl-desc",
            Item::Dagger               => "item-dagger-desc",
            Item::Gloves               => "item-gloves-desc",
            Item::PoisonRing           => "item-poison-ring-desc",
            Item::CastingKnives        => "item-casting-knives-desc",
            Item::Whip                 => "item-whip-desc",
            Item::Priviledge           => "item-priviledge-desc",
            Item::Monocle              => "item-monocle-desc",
            Item::BrokenMirror         => "item-broken-mirror-desc",
            Item::Sextant              => "item-sextant-desc",
            Item::Coat                 => "item-coat-desc",
            Item::Tome                 => "item-tome-desc",
            Item::CoatOfArmorOfTheLoge => "item-coat-of-armor-desc",
        })
    }

    fn tr_emoji(self) -> String {
        lookup_emoji(match self {
            Item::Key                  => "item-key-emoji",
            Item::Goblet               => "item-goblet-emoji",
            Item::BagKey               => "item-bag-key-emoji",
            Item::BagGoblet            => "item-bag-goblet-emoji",
            Item::BlackPearl           => "item-black-pearl-emoji",
            Item::Dagger               => "item-dagger-emoji",
            Item::Gloves               => "item-gloves-emoji",
            Item::PoisonRing           => "item-poison-ring-emoji",
            Item::CastingKnives        => "item-casting-knives-emoji",
            Item::Whip                 => "item-whip-emoji",
            Item::Priviledge           => "item-priviledge-emoji",
            Item::Monocle              => "item-monocle-emoji",
            Item::BrokenMirror         => "item-broken-mirror-emoji",
            Item::Sextant              => "item-sextant-emoji",
            Item::Coat                 => "item-coat-emoji",
            Item::Tome                 => "item-tome-emoji",
            Item::CoatOfArmorOfTheLoge => "item-coat-of-armor-emoji",
        })
    }
}

// ── Job ──────────────────────────────────────────────────────────────────────

impl Translate for Job {
    fn tr_name(self, lang: Lang) -> String {
        lookup(lang, match self {
            Job::Thug        => "job-thug-name",
            Job::GrandMaster => "job-grand-master-name",
            Job::Bodyguard   => "job-bodyguard-name",
            Job::Duelist     => "job-duelist-name",
            Job::PoisonMixer => "job-poison-mixer-name",
            Job::Doctor      => "job-doctor-name",
            Job::Priest      => "job-priest-name",
            Job::Hypnotist   => "job-hypnotist-name",
            Job::Diplomat    => "job-diplomat-name",
            Job::Clairvoyant => "job-clairvoyant-name",
        })
    }

    fn tr_desc(self, lang: Lang) -> String {
        lookup(lang, match self {
            Job::Thug        => "job-thug-desc",
            Job::GrandMaster => "job-grand-master-desc",
            Job::Bodyguard   => "job-bodyguard-desc",
            Job::Duelist     => "job-duelist-desc",
            Job::PoisonMixer => "job-poison-mixer-desc",
            Job::Doctor      => "job-doctor-desc",
            Job::Priest      => "job-priest-desc",
            Job::Hypnotist   => "job-hypnotist-desc",
            Job::Diplomat    => "job-diplomat-desc",
            Job::Clairvoyant => "job-clairvoyant-desc",
        })
    }

    fn tr_emoji(self) -> String {
        lookup_emoji(match self {
            Job::Thug        => "job-thug-emoji",
            Job::GrandMaster => "job-grand-master-emoji",
            Job::Bodyguard   => "job-bodyguard-emoji",
            Job::Duelist     => "job-duelist-emoji",
            Job::PoisonMixer => "job-poison-mixer-emoji",
            Job::Doctor      => "job-doctor-emoji",
            Job::Priest      => "job-priest-emoji",
            Job::Hypnotist   => "job-hypnotist-emoji",
            Job::Diplomat    => "job-diplomat-emoji",
            Job::Clairvoyant => "job-clairvoyant-emoji",
        })
    }
}
