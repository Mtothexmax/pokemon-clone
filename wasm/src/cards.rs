//! The Brick Beast cards (Kaigen `st_cards.c`, the three.js `cards.ts` / `levels.ts`): the card table, the
//! ability kinds and their colours, the zones (locations), the decks, the 18 starter cards, the upgrade
//! stages, the collection in the save, and each card's picture (the Voxelmon's front pixel art of Kaigen
//! `st_beasts_data.c`, in `beasts.bin`, made by `tools/make_beasts.py`).
//!
//! Cards are indices (`usize`, Kaigen `CardId`): `CARDS[c]`. The collection is `GameSave::collection` (a bit
//! per card), the deck `GameSave::deck` (card indices, up to `DECK_SIZE`), the stages `GameSave::card_level`.


use crate::save::GameSave;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Kind {
    None,
    Reveal,
    Ongoing,
    Activate,
    EndTurn,
    Start,
}

pub struct CardDef {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: Kind,
    pub cost: i32,
    pub power: i32,
    pub text: &'static str,
}

const fn c(id: &'static str, name: &'static str, kind: Kind, cost: i32, power: i32, text: &'static str) -> CardDef {
    CardDef { id, name, kind, cost, power, text }
}

use Kind::*;

/// Every card, in Kaigen `CardId` order. The first `CARD_POOL` can be collected; then the special reward
/// (Goldbrick, all golden studs) and the token (Brickling, made by Glimmerling).
pub const CARDS: [CardDef; CARD_COUNT] = [
    c("pebbit", "Pebbit", None, 1, 2, "A sturdy little pebble bunny."),
    c("flicker", "Flicker", Reveal, 1, 1, "On Reveal: Your other cards here get +1 Power."),
    c("drizzlet", "Drizzlet", Reveal, 1, 1, "On Reveal: Draw a card."),
    c("sproutle", "Sproutle", EndTurn, 1, 1, "End of Turn: This card gets +1 Power."),
    c("zapling", "Zapling", Reveal, 1, 2, "On Reveal: A random enemy card here gets -1 Power."),
    c("quickhop", "Quickhop", Start, 1, 2, "Game Start: Put this card into your opening hand."),
    c("voltail", "Voltail", Reveal, 2, 2, "On Reveal: A random enemy card here gets -2 Power."),
    c("cinderpup", "Cinderpup", Reveal, 2, 2, "On Reveal: If an enemy card is here, +3 Power."),
    c("mosscub", "Mosscub", Ongoing, 2, 1, "Ongoing: Your other cards here have +1 Power."),
    c("gloomoth", "Gloomoth", Ongoing, 2, 2, "Ongoing: Enemy cards here have -1 Power."),
    c("gustling", "Gustling", Activate, 2, 2, "Activate: Your other cards here get +1 Power."),
    c("sparkmole", "Sparkmole", Activate, 2, 3, "Activate: Draw a card."),
    c("tickbot", "Tickbot", EndTurn, 2, 2, "End of Turn: A random other card of yours here gets +1 Power."),
    c("tidalotl", "Tidalotl", Reveal, 3, 3, "On Reveal: Your cards in other zones get +1 Power."),
    c("rockodile", "Rockodile", None, 3, 5, "Big jaws, no tricks."),
    c("fungoo", "Fungoo", Reveal, 3, 2, "On Reveal: All enemy cards here get -1 Power."),
    c("zephyrrow", "Zephyrrow", Ongoing, 3, 3, "Ongoing: +4 Power if this is your only card here."),
    c("glimmerling", "Glimmerling", Reveal, 3, 3, "On Reveal: Add a Brickling (1 cost, 2 Power) to your hand."),
    c("echowl", "Echowl", Ongoing, 3, 2, "Ongoing: On Reveal abilities of your cards here happen twice."),
    c("buildbeaver", "Buildbeaver", Start, 3, 3, "Game Start: This card gets +2 Power for this game."),
    c("thundrake", "Thundrake", Reveal, 4, 5, "On Reveal: The strongest enemy card here gets -3 Power."),
    c("coralyth", "Coralyth", Ongoing, 4, 3, "Ongoing: Your cards in other zones have +1 Power."),
    c("emberhorn", "Emberhorn", Reveal, 4, 7, "On Reveal: Your other cards here get -1 Power."),
    c("bramblewolf", "Bramblewolf", Ongoing, 4, 3, "Ongoing: +1 Power for each other card you have here."),
    c("phantomule", "Phantomule", Reveal, 4, 3, "On Reveal: Destroy the weakest enemy card here."),
    c("boomshroom", "Boomshroom", Activate, 4, 4, "Activate: All enemy cards here get -2 Power."),
    c("glaciomoth", "Glaciomoth", Reveal, 5, 4, "On Reveal: All enemy cards here get -2 Power."),
    c("magmaw", "Magmaw", None, 5, 8, "A walking volcano."),
    c("lumifox", "Lumifox", Reveal, 5, 3, "On Reveal: Your other cards here get +2 Power."),
    c("cragoliath", "Cragoliath", Ongoing, 5, 6, "Ongoing: +3 Power if your side of this zone is full."),
    c("clockcrab", "Clockcrab", EndTurn, 5, 4, "End of Turn: This card gets +2 Power."),
    c("titanoak", "Titanoak", None, 6, 11, "An ancient tree-titan."),
    c("stormaw", "Stormaw", Reveal, 6, 7, "On Reveal: ALL enemy cards get -1 Power."),
    c("abyssquid", "Abyssquid", Ongoing, 6, 5, "Ongoing: Your other cards here have +2 Power."),
    c("goldbrick", "Goldbrick", Start, 2, 3, "Game Start: Put this card into your opening hand. It gets +2 Power for this game."),
    c("brickling", "Brickling", None, 1, 2, "A tiny sparkle brick."),
];

pub const CARD_COUNT: usize = 36;
/// The cards you can collect (not the special reward, not the token).
pub const CARD_POOL: usize = 34;
pub const DECK_SIZE: usize = 12;

pub const PEBBIT: usize = 0;
pub const FLICKER: usize = 1;
pub const DRIZZLET: usize = 2;
pub const SPROUTLE: usize = 3;
pub const ZAPLING: usize = 4;
pub const QUICKHOP: usize = 5;
pub const VOLTAIL: usize = 6;
pub const CINDERPUP: usize = 7;
pub const MOSSCUB: usize = 8;
pub const GLOOMOTH: usize = 9;
pub const GUSTLING: usize = 10;
pub const SPARKMOLE: usize = 11;
pub const TICKBOT: usize = 12;
pub const TIDALOTL: usize = 13;
pub const ROCKODILE: usize = 14;
pub const FUNGOO: usize = 15;
pub const ZEPHYRROW: usize = 16;
pub const GLIMMERLING: usize = 17;
pub const ECHOWL: usize = 18;
pub const BUILDBEAVER: usize = 19;
pub const THUNDRAKE: usize = 20;
pub const CORALYTH: usize = 21;
pub const EMBERHORN: usize = 22;
pub const BRAMBLEWOLF: usize = 23;
pub const PHANTOMULE: usize = 24;
pub const BOOMSHROOM: usize = 25;
pub const GLACIOMOTH: usize = 26;
pub const MAGMAW: usize = 27;
pub const LUMIFOX: usize = 28;
pub const CRAGOLIATH: usize = 29;
pub const CLOCKCRAB: usize = 30;
pub const TITANOAK: usize = 31;
pub const STORMAW: usize = 32;
pub const ABYSSQUID: usize = 33;
pub const GOLDBRICK: usize = 34;
pub const BRICKLING: usize = 35;

pub fn card(c: usize) -> &'static CardDef {
    &CARDS[c.min(CARD_COUNT - 1)]
}

/// A card by its id ("pebbit"); `None` if there is no such card.
pub fn by_id(id: &str) -> Option<usize> {
    CARDS.iter().position(|d| d.id == id)
}

/// An ability kind's label and colours (light, dark) as 0xRRGGBB.
pub struct KindInfo {
    pub label: &'static str,
    pub color: u32,
    pub dark: u32,
    /// the symbol on the card (Kaigen glyphs; the three.js version used Material icons)
    pub glyph: &'static str,
}

pub fn kind_info(k: Kind) -> KindInfo {
    let (label, color, dark, glyph) = match k {
        Reveal => ("On Reveal", 0xff8a3d, 0xb8400f, "!"),
        Ongoing => ("Ongoing", 0x3d9bff, 0x1655b0, "\u{221e}"),
        Activate => ("Activate", 0x4fc06a, 0x1e7a35, "\u{25b6}"),
        EndTurn => ("End of Turn", 0x9a74d6, 0x4f2f8a, "\u{21bb}"),
        Start => ("Game Start", 0xffc928, 0xa87a00, "\u{2605}"),
        None => ("No ability", 0xb8bcc0, 0x5c6166, ""),
    };
    KindInfo { label, color, dark, glyph }
}

// ---------------------------------------------------------------- zones (locations)

pub const ZONE_COUNT: usize = 12;

pub struct ZoneDef {
    pub id: &'static str,
    pub name: &'static str,
    pub text: &'static str,
    pub color: u32,
}

/// The locations (Kaigen `zone_def`); their pictures are `assets/zones/<id>.jpg` (400x225, from the three.js
/// `static/zones`), the battle backgrounds `assets/backgrounds/{forest,cave,lab}.png` (480x270 pixel art).
pub const ZONES: [ZoneDef; ZONE_COUNT] = [
    ZoneDef { id: "meadow", name: "Sunny Meadow", text: "No effect. Just a nice day.", color: 0x7ccf5a },
    ZoneDef { id: "stage", name: "Spotlight Stage", text: "On Reveal abilities here happen twice.", color: 0xff8a3d },
    ZoneDef { id: "pond", name: "Calm Pond", text: "Ongoing cards here have +2 Power.", color: 0x3d9bff },
    ZoneDef { id: "grove", name: "Old Grove", text: "Cards with no ability have +2 Power here.", color: 0x3e9a32 },
    ZoneDef { id: "ledge", name: "Sky Ledge", text: "Only 2 cards per side fit here.", color: 0x8fe0e6 },
    ZoneDef { id: "barn", name: "Haunted Barn", text: "All cards here have -1 Power.", color: 0x7a5aa0 },
    ZoneDef { id: "library", name: "Quiet Library", text: "On Reveal abilities do not trigger here.", color: 0xb08a5a },
    ZoneDef { id: "gym", name: "Brick Gym", text: "All cards here have +1 Power.", color: 0xe04040 },
    ZoneDef { id: "toybox", name: "Toy Box", text: "Cards that cost 1 or 2 have +2 Power here.", color: 0xffb030 },
    ZoneDef { id: "clock", name: "Clock Tower", text: "End of Turn abilities here happen twice.", color: 0x9a74d6 },
    ZoneDef { id: "tower", name: "Thunder Tower", text: "Closed on the final turn.", color: 0xffd23d },
    ZoneDef { id: "workshop", name: "Workshop", text: "Activate cards here have +2 Power.", color: 0x4fc06a },
];

pub fn zone_picture(z: usize) -> String {
    format!("assets/zones/{}.jpg", ZONES[z % ZONE_COUNT].id)
}

// ---------------------------------------------------------------- decks

pub const DECK_STARTER: usize = 0;
pub const DECK_RIVAL: usize = 1;
pub const DECK_RIVAL_2: usize = 2;
pub const DECK_PRACTICE: usize = 3;
/// Pip: stronger than Dex's rematch deck (and every card +1 Power in the battle).
pub const DECK_PIP: usize = 4;

pub const DECKS: [[usize; DECK_SIZE]; 5] = [
    [QUICKHOP, FLICKER, DRIZZLET, ZAPLING, VOLTAIL, CINDERPUP, GUSTLING, TIDALOTL, ECHOWL, THUNDRAKE, EMBERHORN, TITANOAK],
    [FLICKER, ZAPLING, CINDERPUP, GLOOMOTH, SPARKMOLE, TICKBOT, TIDALOTL, PHANTOMULE, THUNDRAKE, BOOMSHROOM, LUMIFOX, STORMAW],
    [FLICKER, VOLTAIL, CINDERPUP, TICKBOT, ECHOWL, CORALYTH, THUNDRAKE, PHANTOMULE, GLACIOMOTH, LUMIFOX, STORMAW, ABYSSQUID],
    [PEBBIT, SPROUTLE, QUICKHOP, MOSSCUB, GUSTLING, ROCKODILE, BUILDBEAVER, BRAMBLEWOLF, CORALYTH, CLOCKCRAB, CRAGOLIATH, TITANOAK],
    [QUICKHOP, ZAPLING, VOLTAIL, CINDERPUP, GLOOMOTH, TIDALOTL, THUNDRAKE, PHANTOMULE, GLACIOMOTH, MAGMAW, STORMAW, ABYSSQUID],
];

/// The 18 cards Prof. Quill gives you.
pub const STARTER: [usize; 18] = [
    PEBBIT, FLICKER, DRIZZLET, SPROUTLE, ZAPLING, QUICKHOP, VOLTAIL, CINDERPUP, MOSSCUB, GUSTLING, TIDALOTL, ROCKODILE, FUNGOO, THUNDRAKE,
    EMBERHORN, BRAMBLEWOLF, TITANOAK, ECHOWL,
];

// ---------------------------------------------------------------- upgrade stages (levels.ts)

pub const VOXEL_LEVEL: u8 = 4;
pub const WILD_EXP: u32 = 20;

pub struct StageDef {
    pub name: &'static str,
    /// EXP from the stage before
    pub cost: u32,
    pub blurb: &'static str,
    pub frame: u32,
}

pub fn stage(s: u8) -> StageDef {
    let (name, cost, blurb, frame) = match s {
        1 => ("Bronze", 15, "A shiny bronze frame.", 0xcd7f32),
        2 => ("Silver", 30, "Silver frame with glitter.", 0xc8d0d8),
        3 => ("Gold", 45, "Gold frame and rainbow holo foil.", 0xffcc33),
        4 => ("Voxelmon", 80, "The beast steps out of the card: a real 3D Voxelmon!", 0xa05aff),
        _ => ("Basic", 0, "A fresh card.", 0x282c34),
    };
    StageDef { name, cost, blurb, frame }
}

/// Only 6-energy cards become a Voxelmon.
pub fn can_become_voxel(c: usize) -> bool {
    card(c).cost >= 6
}
pub fn max_level(c: usize) -> u8 {
    if can_become_voxel(c) {
        VOXEL_LEVEL
    } else {
        VOXEL_LEVEL - 1
    }
}

/// EXP for a card battle (result 0 win, 1 lose, 2 draw, else retreat): a win is worth the most.
pub fn battle_exp(result: i32, rival: bool) -> u32 {
    match result {
        0 => {
            if rival {
                40
            } else {
                30
            }
        }
        2 => 15,
        1 => 10,
        _ => 5,
    }
}

// ---------------------------------------------------------------- the collection in the save

pub fn owns(s: &GameSave, c: usize) -> bool {
    c < 64 && (s.collection >> c) & 1 == 1
}
pub fn owned_count(s: &GameSave) -> u32 {
    (0..CARD_POOL).filter(|&c| owns(s, c)).count() as u32
}
pub fn give_card(s: &mut GameSave, c: usize) {
    s.collection |= 1u64 << c;
}
pub fn level(s: &GameSave, c: usize) -> u8 {
    s.card_level.get(c).copied().unwrap_or(0)
}
pub fn in_deck(s: &GameSave, c: usize) -> bool {
    s.deck.iter().any(|&d| d as usize == c)
}

/// Prof. Quill's gift: the 18 starter cards and the starter deck.
pub fn give_starter(s: &mut GameSave) {
    for c in STARTER {
        give_card(s, c);
    }
    s.deck = DECKS[DECK_STARTER].iter().map(|&c| c as u8).collect();
    s.has_deck = true;
}

/// A random card the player does not have yet (rare: cost 5+ first), given; `None` when the collection is
/// complete (Kaigen `reward_card`, same generator).
pub fn reward_card(s: &mut GameSave, seed: &mut u32, rare: bool) -> Option<usize> {
    let missing: Vec<usize> = (0..CARD_POOL).filter(|&c| !owns(s, c)).collect();
    if missing.is_empty() {
        return Option::None;
    }
    let rares: Vec<usize> = missing.iter().copied().filter(|&c| card(c).cost >= 5).collect();
    let r = crate::lcg(seed) as usize;
    let c = if rare && !rares.is_empty() { rares[r % rares.len()] } else { missing[r % missing.len()] };
    give_card(s, c);
    Some(c)
}

/// EXP for the next stage, 0 at the last one.
pub fn upgrade_cost(s: &GameSave, c: usize) -> u32 {
    let lv = level(s, c);
    if lv >= max_level(c) {
        0
    } else {
        stage(lv + 1).cost
    }
}

/// Spends EXP on the next stage; the first Voxelmon walks with you at once.
pub fn upgrade(s: &mut GameSave, c: usize) -> bool {
    let cost = upgrade_cost(s, c);
    if cost == 0 || s.exp < cost {
        return false;
    }
    s.exp -= cost;
    if s.card_level.len() < CARD_COUNT {
        s.card_level.resize(CARD_COUNT, 0);
    }
    s.card_level[c] += 1;
    if s.card_level[c] == VOXEL_LEVEL && s.voxelmon == 0 {
        s.voxelmon = c as i32 + 1;
    }
    true
}

/// Foil packs of rare cards are gold.
pub fn pack_rare(c: usize) -> bool {
    card(c).cost >= 5 || c == GOLDBRICK
}

// ---------------------------------------------------------------- the pictures

static BEASTS: &[u8] = include_bytes!("beasts.bin");

/// A card's picture: width, height and RGBA pixels (top row first), the beast's front in pixel art.
pub fn picture(c: usize) -> (u32, u32, &'static [u8]) {
    let mut o = 0usize;
    for i in 0..CARD_COUNT {
        let (w, h) = (BEASTS[o] as usize, BEASTS[o + 1] as usize);
        if i == c.min(CARD_COUNT - 1) {
            return (w as u32, h as u32, &BEASTS[o + 2..o + 2 + w * h * 4]);
        }
        o += 2 + w * h * 4;
    }
    (0, 0, &[])
}
