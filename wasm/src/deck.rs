//! The card screens outside battles (Kaigen `st_cardui.c` deck_* / up_* / rewards_*, `st_cardgame.c`): the
//! deck screen (C or the Deck button: the collection with the cards you do not have as dark "???", the 12-card
//! deck strip, the picked card big with what you can do: upgrade with EXP, walk with a Voxelmon, add to /
//! remove from the deck; a double click puts a card into the deck), the upgrade moment (the card flips into its
//! new frame; a 6-energy card bursts into voxels), and the rewards as sealed foil packs to tear open.
//!
//! It is a 2D HTML menu: every card is the PNG of `cardface.rs` (the same picture the 3D battle puts on its
//! card meshes) as an `<img>`. Slots: `deck_head`, `deck_coll` (the scrolling collection, rebuilt only when
//! a card's picture changes, so it keeps its scroll), `deck_marks` (a style slot: the picked card lifted, the
//! cards in your deck framed green with a tick), `deck_side` (the side panel, or the sheet on phones),
//! `deck_strip`, `deck_btn` (the HUD button), `packs`, `upgrade`. Every button is a `data-action` (phones).

use crate::esc;
use crate::cardface::{CardFaces, Face};
use crate::cards::{self, CARD_COUNT, CARD_POOL, DECK_SIZE, GOLDBRICK, VOXEL_LEVEL};
use crate::dialog::DialogId;
use crate::save::Save;
use crate::sound::{Sfx, Sound};
use engine::prelude::*;
use serde_json::{json, Value};

/// A pack nobody has opened yet.
const PACK_SEALED: f32 = -100.0;
/// The card is out (the strip is off, the halves fly).
const PACK_REVEAL: f32 = 0.42;
/// The foil is gone.
const PACK_GONE: f32 = 1.1;
/// The upgrade moment: the card turns edge-on, comes back in its new frame; a Voxelmon bursts; the button.
const UP_SWAP: f32 = 1.3;
const UP_BURST: f32 = 1.25;
const UP_DONE: f32 = 2.0;
/// Two clicks on the same collection card within this time put it into the deck.
const DOUBLE_CLICK: f32 = 0.45;

/// What the card screens hand back to the play screen.
#[derive(Default)]
pub struct CardsOut {
    /// the rewards were closed: open this dialogue (Kaigen `cg->after`)
    pub after: Option<DialogId>,
}

pub struct Upgrade {
    pub card: usize,
    pub from: u8,
    pub to: u8,
    pub t: f32,
}

#[derive(Default)]
pub struct CardUi {
    pub deck_open: bool,
    /// the card shown big
    pub pick: Option<usize>,
    /// "Add to deck" with a full deck: the picked card waits for the deck card it replaces (a tap on one in the
    /// chooser or the strip swaps them; Cancel, Esc / C or picking another card drops it)
    pub replacing: Option<usize>,
    last_coll: Option<(usize, f32)>,
    time: f32,
    // ---- rewards ----
    pub rewards: Vec<usize>,
    title: String,
    reward_t: f32,
    /// seconds since each pack was torn open (`PACK_SEALED`: not yet; < 0: about to, "Reveal all")
    tear: [f32; 3],
    after: Option<DialogId>,
    /// the packs' HTML is rebuilt only when their state changes (CSS animations run with negative delays)
    pack_key: String,
    // ---- the upgrade moment ----
    pub up: Option<Upgrade>,
    up_key: String,
    /// the narrow layout was used last (phones held upright)
    narrow: bool,
    /// card faces asked for while the deck was open (tests)
    pub upgrades: u32,
    pub adds: u32,
}

impl CardUi {
    pub fn new() -> Self {
        CardUi { tear: [PACK_SEALED; 3], ..Default::default() }
    }

    /// The card screens own the screen (no walking, no talking).
    pub fn busy(&self) -> bool {
        self.deck_open || !self.rewards.is_empty()
    }

    /// Sealed foil packs to tear open, one per card (at most 3); `after` opens when they are closed.
    pub fn show_rewards(&mut self, title: &str, ids: &[usize], after: Option<DialogId>) {
        self.rewards = ids.iter().copied().take(3).collect();
        self.title = title.to_string();
        self.reward_t = 0.0;
        self.tear = [PACK_SEALED; 3];
        self.after = after;
        self.pack_key.clear();
    }

    pub fn open_deck(&mut self, sound: &mut Sound, ctx: &mut Ctx) {
        if !self.deck_open {
            self.deck_open = true;
            sound.sfx(ctx, Sfx::Blip, 1.4, 0.4);
        }
    }

    /// Every frame (Kaigen `cg_update` without the battle): the packs, the deck screen's clicks and keys, the
    /// upgrade moment's sounds, the legendary card. `world_free`: nothing else holds the screen (C may open).
    pub fn update(&mut self, ctx: &mut Ctx, save: &mut Save, sound: &mut Sound, actions: &[String], world_free: bool, dt: f32) -> CardsOut {
        let mut out = CardsOut::default();
        self.time += dt;
        let key = |k: &str| ctx.input.key_pressed(k);
        let (k_enter, k_e, k_space, k_esc, k_c) = (key("Enter"), key("KeyE"), key("Space"), key("Escape"), key("KeyC"));
        // the upgrade moment's sounds: the flip (or the burst), then a jingle
        if let Some(u) = &mut self.up {
            let (t0, t1) = (u.t, u.t + dt);
            u.t = t1;
            let vox = u.to >= VOXEL_LEVEL;
            if t0 < 0.3 && t1 >= 0.3 {
                sound.sfx(ctx, Sfx::Blip, 0.6, 0.4);
            }
            if !vox && t0 < UP_SWAP && t1 >= UP_SWAP {
                sound.sfx(ctx, Sfx::Pop, 1.2, 0.8);
                sound.sfx(ctx, Sfx::Coin, 1.0, 0.8);
            }
            if vox && t0 < UP_BURST && t1 >= UP_BURST {
                sound.sfx(ctx, Sfx::Kick, 0.5, 1.0);
                sound.sfx(ctx, Sfx::Fanfare, 1.0, 0.9);
            }
        }
        // the legendary card for all golden studs
        let s = &mut save.data;
        if s.legendary && !cards::owns(s, GOLDBRICK) && !self.busy() && world_free {
            cards::give_card(s, GOLDBRICK);
            save.mark();
            self.show_rewards("All golden studs found! A legendary card appears\u{2026}", &[GOLDBRICK], None);
        }
        if !self.rewards.is_empty() {
            self.rewards_update(ctx, sound, actions, k_enter || k_e || k_space, dt, &mut out);
            return out;
        }
        if self.deck_open {
            if self.deck_input(ctx, save, sound, actions, k_enter || k_space || k_e || k_esc || k_c, k_c || k_esc) {
                self.deck_open = false;
                self.pick = None;
                self.replacing = None;
                sound.sfx(ctx, Sfx::Blip, 1.2, 0.3);
            }
            return out;
        }
        // C or the Deck button opens the deck screen once you have cards
        let s = &save.data;
        if s.has_deck && world_free && (k_c || actions.iter().any(|a| a == "deck")) {
            self.open_deck(sound, ctx);
        }
        out
    }

    fn rewards_update(&mut self, ctx: &mut Ctx, sound: &mut Sound, actions: &[String], mut keys: bool, dt: f32, out: &mut CardsOut) {
        // a tap tears a pack open (a crinkle; the card pops out at PACK_REVEAL with a jingle), "Reveal all" or a
        // key opens the rest one after another; once every card is out, the same keys or "Awesome!" close it
        self.reward_t += dt;
        let n = self.rewards.len();
        let mut all = true;
        for i in 0..n {
            let t = &mut self.tear[i];
            if *t > PACK_SEALED + 1.0 {
                let before = *t;
                *t += dt;
                if before < 0.0 && *t >= 0.0 && i > 0 {
                    sound.sfx(ctx, Sfx::Splash, 1.9, 0.35);
                }
                if before < PACK_REVEAL && *t >= PACK_REVEAL {
                    let rare = cards::pack_rare(self.rewards[i]);
                    sound.sfx(ctx, if rare { Sfx::Fanfare } else { Sfx::Coin }, 1.0 + 0.12 * i as f32, 0.7);
                }
            }
            all &= self.tear[i] >= PACK_REVEAL;
        }
        let pack = actions.iter().find_map(|a| a.strip_prefix("pack:").and_then(|n| n.parse::<usize>().ok()));
        let reveal_all = actions.iter().any(|a| a == "pack_all");
        if let Some(i) = pack.filter(|&i| i < n && self.tear[i] <= PACK_SEALED + 1.0) {
            self.tear[i] = 0.0;
            sound.sfx(ctx, Sfx::Splash, 1.9, 0.45);
        } else if !all && (reveal_all || keys) {
            let mut k = 0;
            for i in 0..n {
                if self.tear[i] <= PACK_SEALED + 1.0 {
                    self.tear[i] = -0.26 * k as f32;
                    k += 1;
                }
            }
            sound.sfx(ctx, Sfx::Splash, 1.9, 0.45);
            keys = false;
        }
        if all && (keys || actions.iter().any(|a| a == "pack_ok")) {
            self.rewards.clear();
            self.pack_key.clear();
            sound.sfx(ctx, Sfx::Blip, 1.2, 0.4);
            out.after = self.after.take();
        }
    }

    /// The deck screen's clicks; returns true when it was closed.
    fn deck_input(&mut self, ctx: &mut Ctx, save: &mut Save, sound: &mut Sound, actions: &[String], up_keys: bool, close_keys: bool) -> bool {
        if let Some(u) = &self.up {
            // the upgrade moment owns the screen: once done, a tap, its button or a key goes back to the deck
            let done = u.t > if u.to >= VOXEL_LEVEL { UP_DONE + 0.7 } else { UP_DONE };
            if done && (up_keys || actions.iter().any(|a| a == "up_ok")) {
                self.up = None;
                self.up_key.clear();
            }
            return false;
        }
        let s = &mut save.data;
        let mut add = false;
        for a in actions {
            if let Some(c) = a.strip_prefix("coll:").and_then(|n| n.parse::<usize>().ok()) {
                if !cards::owns(s, c) {
                    continue;
                }
                if self.pick != Some(c) {
                    self.replacing = None;
                }
                self.pick = Some(c);
                sound.sfx(ctx, Sfx::Blip, 1.5, 0.25);
                match self.last_coll {
                    Some((l, t)) if l == c && self.time - t < DOUBLE_CLICK => {
                        // a double click: into the deck, if there is room
                        add = !cards::in_deck(s, c);
                        self.last_coll = None;
                    }
                    _ => self.last_coll = Some((c, self.time)),
                }
            } else if let Some(i) = a.strip_prefix("slot:").or_else(|| a.strip_prefix("replace:")).and_then(|n| n.parse::<usize>().ok()) {
                if let (Some(new), Some(old)) = (self.replacing, s.deck.get(i).copied()) {
                    // the swap: the new card takes the old one's place in the deck (the order stays)
                    if !cards::in_deck(s, new) {
                        s.deck[i] = new as u8;
                        self.adds += 1;
                        save.mark();
                        sound.sfx(ctx, Sfx::Pop, 1.3, 0.5);
                        log_info!("deck: {} replaces {}", cards::card(new).id, cards::card(old as usize).id);
                    }
                    self.replacing = None;
                    self.last_coll = None;
                    return false;
                }
                if let Some(&c) = s.deck.get(i) {
                    self.pick = Some(c as usize);
                    sound.sfx(ctx, Sfx::Blip, 1.5, 0.25);
                }
            }
        }
        let has = |a: &str| actions.iter().any(|x| x == a);
        if self.replacing.is_some() && (has("deck_cancel") || close_keys) {
            // Esc / C first drop the swap, a second press closes the screen
            self.replacing = None;
            sound.sfx(ctx, Sfx::Blip, 1.2, 0.3);
            return has("deck_close");
        }
        if let Some(c) = self.pick {
            if (add || has("deck_add")) && !cards::in_deck(s, c) {
                if s.deck.len() < DECK_SIZE {
                    s.deck.push(c as u8);
                    self.adds += 1;
                    save.mark();
                    sound.sfx(ctx, Sfx::Pop, 1.3, 0.5);
                } else {
                    // full: ask which deck card it replaces
                    self.replacing = Some(c);
                    sound.sfx(ctx, Sfx::Blip, 1.3, 0.3);
                }
            }
            let s = &mut save.data;
            if has("deck_remove") && cards::in_deck(s, c) {
                s.deck.retain(|&d| d as usize != c);
                save.mark();
                sound.sfx(ctx, Sfx::Pop, 0.8, 0.5);
            }
            let s = &mut save.data;
            if has("deck_upgrade") && cards::upgrade(s, c) {
                let lv = cards::level(s, c);
                self.up = Some(Upgrade { card: c, from: lv - 1, to: lv, t: 0.0 });
                self.up_key.clear();
                self.upgrades += 1;
                save.mark();
            }
            let s = &mut save.data;
            if has("deck_walk") && cards::level(s, c) >= VOXEL_LEVEL {
                s.voxelmon = c as i32 + 1;
                save.mark();
            }
        }
        if has("deck_unpick") {
            self.pick = None; // the sheet's x (phones)
            self.replacing = None;
        }
        has("deck_close") || close_keys
    }

    // ------------------------------------------------------------------ drawing

    /// The deck screen, the HUD button, the packs and the upgrade moment. `hud`: the Deck button may show.
    pub fn draw(&mut self, ctx: &mut Ctx, save: &Save, faces: &mut CardFaces, hud: bool, touch: bool) {
        let s = &save.data;
        let (sw, sh) = (ctx.canvas.client_width().max(1) as f32, ctx.canvas.client_height().max(1) as f32);
        let narrow = sw < 640.0;
        self.narrow = narrow;
        // the Deck button, top right left of the menu button (narrow screens: below it)
        let btn = if hud && s.has_deck && !self.busy() {
            let label = if narrow {
                // a compact button under the menu button: the goal pill (up to 100vw - 90 px) never reaches it
                format!("<b>Deck</b><small>{}</small>", cards::owned_count(s))
            } else {
                format!("Deck{}&nbsp; {} cards &middot; {} EXP", if touch { "" } else { " (C)" }, cards::owned_count(s), s.exp)
            };
            format!("<button class='bh-deckbtn{}' data-action='deck'>{label}</button>", if narrow { " narrow" } else { "" })
        } else {
            String::new()
        };
        ctx.ui.set("deck_btn", &btn);
        if self.deck_open && !self.up.as_ref().is_some_and(|u| u.to >= VOXEL_LEVEL && u.t > UP_BURST + 0.3) {
            self.draw_deck(ctx, s, faces, sw, sh, narrow, touch);
        } else {
            for slot in ["deck_head", "deck_coll", "deck_marks", "deck_side", "deck_strip"] {
                ctx.ui.set(slot, "");
            }
        }
        self.draw_upgrade(ctx, faces, sw, sh);
        self.draw_packs(ctx, s, faces, sw);
    }

    fn img(faces: &mut CardFaces, f: Face) -> String {
        match faces.url(f) {
            Some(u) => format!("<img src='{u}' alt='' draggable='false'>"),
            None => String::new(),
        }
    }

    /// A card in the DOM: its face as a picture, glows for the stages (the face carries the rest).
    fn card_html(faces: &mut CardFaces, f: Face, w: f32, action: &str, id: &str) -> String {
        let cls = match f.level {
            _ if f.unknown => " unk",
            4 => " lv4",
            3 => " lv3",
            _ => "",
        };
        let act = if action.is_empty() { String::new() } else { format!(" data-action='{action}'") };
        let id = if id.is_empty() { String::new() } else { format!(" id='{id}'") };
        format!("<div class='bc{cls}'{act}{id} style='width:{w:.0}px;height:{:.0}px'>{}</div>", w * 1.4, Self::img(faces, f))
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_deck(&mut self, ctx: &mut Ctx, s: &crate::save::GameSave, faces: &mut CardFaces, sw: f32, sh: f32, narrow: bool, touch: bool) {
        let pad = 12.0;
        let side_w = if narrow { 0.0 } else { (sw * 0.32).min(280.0) };
        let coll_w = sw - pad * 2.0 - if narrow { 0.0 } else { side_w + 12.0 };
        let per_row = if narrow { 4.0 } else { ((coll_w + 8.0) / 92.0).floor().max(4.0) };
        let gw = ((coll_w - 16.0 - 8.0 * (per_row - 1.0)) / per_row - 1.0).clamp(52.0, 84.0);
        let cols = if narrow { 6.0 } else { 12.0 };
        let strip_inner = sw - pad * 2.0 - 16.0;
        let sw_card = (if narrow { 80.0f32 } else { 64.0 }).min((strip_inner - 6.0 * (cols - 1.0)) / cols).floor();
        let rows = (DECK_SIZE as f32 / cols).ceil();
        let head_h = if narrow { 66.0 } else { 74.0 };
        let strip_h = 8.0 + rows * sw_card * 1.4 + (rows - 1.0) * 6.0 + 12.0;
        // ---- the header: title, counts, the way out ----
        let full = s.deck.len() == DECK_SIZE;
        ctx.ui.set(
            "deck_head",
            &format!(
                "<div class='dk-head' style='height:{head_h}px'><div class='dk-title'><b>Your cards</b><div class='dk-counts'><span>{} / {CARD_POOL} collected</span><span class='{}'>Deck {} / {DECK_SIZE}</span><span class='dk-exp'>{} EXP</span></div></div>\
                 <button class='dk-close' data-action='deck_close'><b>&times;</b> {}</button></div>",
                cards::owned_count(s),
                if full { "dk-ok" } else { "dk-bad" },
                s.deck.len(),
                s.exp,
                if narrow || touch { "Close" } else { "Close (C)" }
            ),
        );
        // ---- the collection: every card, the ones you do not have as dark "???" (Goldbrick only once owned) ----
        let mut grid = String::new();
        for c in 0..CARD_COUNT - 1 {
            let have = cards::owns(s, c);
            if c == GOLDBRICK && !have {
                continue;
            }
            let f = if have { Face::new(c, cards::level(s, c)) } else { Face::new(c, 0).unknown() };
            grid += &Self::card_html(faces, f, gw, &format!("coll:{c}"), &format!("bc{c}"));
        }
        ctx.ui.set(
            "deck_coll",
            &format!(
                "<div class='dk-coll' style='top:{head_h}px;bottom:{strip_h}px;right:{}px'><div class='dk-cap'>{}</div><div class='dk-grid'>{grid}</div><div class='dk-pad'></div></div>",
                if narrow { 0.0 } else { side_w + pad + 12.0 },
                if touch { "Collection (tap a card, tap twice to add it)" } else { "Collection (click a card, double-click to add it)" }
            ),
        );
        // ---- marks: the picked card lifted, your deck's cards framed green with a tick ----
        let mut marks = String::from("<style>");
        if let Some(c) = self.pick {
            marks += &format!("#ui #bc{c}{{transform:translateY(-14px) scale(1.06);box-shadow:0 0 18px 6px #ffd23d,0 4px 8px rgba(0,0,0,.35);outline:3px solid #fff;z-index:3}}");
            if narrow {
                marks += &format!("#ui .dk-pad{{height:{:.0}px}}", sh * 0.4);
            }
        }
        let ids: Vec<String> = s.deck.iter().map(|c| format!("#ui #bc{c}")).collect();
        if !ids.is_empty() {
            marks += &format!("{}{{--in:1}}", ids.join(","));
            let pseudo = |p: &str| s.deck.iter().map(|c| format!("#ui #bc{c}{p}")).collect::<Vec<_>>().join(",");
            marks += &format!("{}{{content:'';position:absolute;inset:-3px;border:3px solid #27b05a;border-radius:9%/6.5%;pointer-events:none}}", pseudo("::before"));
            let d = (gw * 0.3).clamp(18.0, 26.0);
            marks += &format!(
                "{}{{content:'\\2713';position:absolute;right:{:.0}px;bottom:{:.0}px;width:{d:.0}px;height:{d:.0}px;border-radius:50%;background:#27b05a;border:2px solid #fff;box-sizing:border-box;color:#fff;font-weight:800;font-size:{:.0}px;line-height:{:.0}px;text-align:center;box-shadow:0 1px 3px rgba(0,0,0,.35);pointer-events:none}}",
                pseudo("::after"),
                -d * 0.25,
                // above the face's bottom banner (a quarter of the card's width high), not over the name
                gw * 0.26,
                d * 0.62,
                d - 4.0
            );
        }
        if self.replacing.is_some() {
            // choosing the card to replace: the strip's cards can be tapped too
            marks += "#ui .dk-strip .bc{outline:3px dashed #ffd23d;outline-offset:1px;cursor:pointer}";
        }
        marks += "</style>";
        ctx.ui.set("deck_marks", &marks);
        // ---- the deck strip: 12 slots (two rows of six on phones) ----
        let mut strip = String::new();
        for row in 0..rows as usize {
            strip += "<div class='dk-row'>";
            for i in row * cols as usize..((row + 1) * cols as usize).min(DECK_SIZE) {
                match s.deck.get(i) {
                    Some(&c) => {
                        let c = c as usize;
                        let lit = if self.pick == Some(c) { " lit" } else { "" };
                        strip += &format!("<div class='dk-slot{lit}'>{}</div>", Self::card_html(faces, Face::new(c, cards::level(s, c)), sw_card, &format!("slot:{i}"), ""));
                    }
                    None => strip += &format!("<div class='dk-empty' style='width:{sw_card:.0}px;height:{:.0}px'></div>", sw_card * 1.4),
                }
            }
            strip += "</div>";
        }
        ctx.ui.set("deck_strip", &format!("<div class='dk-strip' style='height:{strip_h}px'>{strip}</div>"));
        // ---- the picked card: big with what you can do (side panel; phones: a sheet from the bottom) ----
        let side = match self.pick {
            Some(c) if self.replacing == Some(c) => {
                // the chooser: the deck's cards to tap, Cancel
                let cw = if narrow { ((sw - 28.0 - 5.0 * 6.0) / 6.0).min(56.0) } else { ((side_w - 24.0 - 3.0 * 6.0) / 4.0).min(64.0) };
                let mut grid = String::new();
                for (i, &d) in s.deck.iter().enumerate() {
                    let d = d as usize;
                    grid += &Self::card_html(faces, Face::new(d, cards::level(s, d)), cw, &format!("replace:{i}"), "");
                }
                let (cls, style) = if narrow {
                    ("dk-sheet dk-repl", String::new())
                } else {
                    ("dk-side dk-repl", format!(" style='top:{}px;bottom:{}px;width:{side_w}px'", head_h + 8.0, strip_h + 8.0))
                };
                format!(
                    "<div class='{cls}'{style}><div class='dk-sheethead'><b>The deck is full</b></div><p>Which card should <b>{}</b> replace?</p><div class='dk-rgrid'>{grid}</div>                     <button class='dk-btn' data-action='deck_cancel' style='background:#5a5e64'>Cancel</button></div>",
                    esc(cards::card(c).name)
                )
            }
            Some(c) if narrow => format!(
                "<div class='dk-sheet'>{}<div class='dk-acts'><div class='dk-sheethead'><b>{}</b><button class='dk-x' data-action='deck_unpick'>&times;</button></div>{}</div></div>",
                Self::card_html(faces, Face::new(c, cards::level(s, c)).big(), (sw * 0.3).min(120.0), "", "dk-big"),
                cards::card(c).name,
                Self::actions_html(s, c)
            ),
            Some(c) => format!(
                "<div class='dk-side' style='top:{}px;bottom:{}px;width:{side_w}px'>{}{}</div>",
                head_h + 8.0,
                strip_h + 8.0,
                Self::card_html(faces, Face::new(c, cards::level(s, c)).big(), (side_w * 0.62).min(170.0), "", "dk-big"),
                Self::actions_html(s, c)
            ),
            None if narrow => String::new(),
            None => format!(
                "<div class='dk-side' style='top:{}px;bottom:{}px;width:{side_w}px'><p>Pick a card on the left to see it big, upgrade it with EXP or change your deck.</p></div>",
                head_h + 8.0,
                strip_h + 8.0
            ),
        };
        ctx.ui.set("deck_side", &side);
    }

    /// The picked card: its text, stage and what you can do with it.
    fn actions_html(s: &crate::save::GameSave, c: usize) -> String {
        let d = cards::card(c);
        let lv = cards::level(s, c);
        let st = cards::stage(lv);
        let mut h = format!("<div class='dk-text'>{}</div><div class='dk-stage'>Stage: {} &mdash; {}</div>", esc(d.text), st.name, st.blurb);
        let cost = cards::upgrade_cost(s, c);
        let btn = |action: &str, label: &str, sub: &str, bg: &str, on: bool| {
            let sub = if sub.is_empty() { String::new() } else { format!("<small>{sub}</small>") };
            format!("<button class='dk-btn' data-action='{}' style='background:{bg}'{}>{label}{sub}</button>", if on { action } else { "" }, if on { "" } else { " disabled" })
        };
        if cost > 0 {
            let sub = if s.exp >= cost { format!("{cost} EXP") } else { format!("{cost} EXP (you have {})", s.exp) };
            h += &btn("deck_upgrade", &format!("Upgrade to {}", cards::stage(lv + 1).name), &sub, "#783cc8", s.exp >= cost);
        } else if !cards::can_become_voxel(c) {
            h += "<div class='dk-stage'>Final stage. Only 6-energy cards become a Voxelmon.</div>";
        }
        if lv >= VOXEL_LEVEL {
            let walking = s.voxelmon == c as i32 + 1;
            h += &btn("deck_walk", if walking { "Walks with you" } else { "Walk with me" }, "", "#3ca050", !walking);
        }
        if cards::in_deck(s, c) {
            h += &btn("deck_remove", "Remove from deck", "", "#b44632", true);
        } else {
            // a full deck still takes it: the button then asks which card it replaces
            h += &btn("deck_add", "Add to deck", if s.deck.len() < DECK_SIZE { "" } else { "The deck is full: pick one to replace" }, "#0055bf", true);
        }
        h
    }

    /// The upgrade moment: built once per upgrade, its timeline runs in CSS (the sounds in `update`).
    fn draw_upgrade(&mut self, ctx: &mut Ctx, faces: &mut CardFaces, sw: f32, sh: f32) {
        let Some(u) = &self.up else {
            ctx.ui.set("upgrade", "");
            return;
        };
        let (c, from, to) = (u.card, u.from, u.to);
        let vox = to >= VOXEL_LEVEL;
        let before = faces.url(Face::new(c, if vox { VOXEL_LEVEL - 1 } else { from }).big());
        let after = faces.url(Face::new(c, to).big());
        let key = format!("{c}-{to}-{}-{}", before.is_some(), after.is_some());
        if key == self.up_key {
            return;
        }
        self.up_key = key;
        let t = u.t;
        let st = cards::stage(to);
        let frame = format!("#{:06x}", st.frame);
        let cw = (sw * 0.42).min(sh * 0.4 / 1.4).min(230.0);
        let d = cards::card(c);
        let img = |u: &Option<String>| u.as_ref().map(|u| format!("<img src='{u}' alt=''>")).unwrap_or_default();
        let rays: String = (0..6).map(|i| format!("<i style='--a:{}deg'></i>", i * 30)).collect();
        let sparks: String = (0..16).map(|i| format!("<b style='--a:{}rad;--d:{:.2}s;--c:{}'></b>", i as f32 * 0.3927, (i % 6) as f32 * 0.04, if i % 3 != 0 { "#fff" } else { &frame })).collect();
        let (title, note, ok) = if vox {
            (format!("{} is a real Voxelmon now!", d.name), "It walks with you. Show it to Prof. Quill at the village exit!".to_string(), "Let's go!")
        } else {
            (format!("{}!", st.name), format!("{} &middot; {}", d.name, st.blurb), "Nice!")
        };
        let cubes: String = if vox {
            const PAL: [u32; 6] = [0xa05aff, 0xffd23d, 0xffffff, 0x3d9bff, 0xff6a5a, 0x5ccb4c];
            (0..48)
                .map(|i| {
                    let h1 = hash(i as u32);
                    let h2 = hash(i as u32 * 7 + 3);
                    let (gx, gy) = ((i % 6) as f32 / 5.0 - 0.5, (i / 6) as f32 / 7.0 - 0.5);
                    let (vx, vy) = (gx * 2.2 + (h1 - 0.5) * 1.4, gy * 2.2 - 0.6 - h2 * 0.8);
                    format!(
                        "<s style='--x:{:.0}px;--y:{:.0}px;--vx:{:.0}px;--vy:{:.0}px;--g:{:.0}px;--r:{:.0}deg;--s:{:.0}px;--c:#{:06x}'></s>",
                        gx * cw,
                        gy * cw * 1.4,
                        vx * cw * 1.6,
                        vy * cw * 1.6,
                        cw * 1.4,
                        (h1 - 0.5) * 720.0,
                        cw * (0.13 + 0.05 * h2),
                        PAL[i % 6]
                    )
                })
                .collect()
        } else {
            String::new()
        };
        ctx.ui.set(
            "upgrade",
            &format!(
                "<div class='up{}' data-action='up_ok' style='--t:-{t:.2}s;--f:{frame};--cw:{cw:.0}px'>\
                 <div class='up-rays'>{rays}</div><div class='up-sparks'>{sparks}</div>\
                 <div class='up-card'><div class='up-old'>{}</div><div class='up-new'>{}</div></div><div class='up-cubes'>{cubes}</div>\
                 <div class='up-flash'></div><div class='up-words'><b>{title}</b><span>{note}</span></div>\
                 <button class='up-ok' data-action='up_ok'>{ok}</button></div>",
                if vox { " vox" } else { "" },
                img(&before),
                img(&after)
            ),
        );
    }

    /// The rewards: a panel with the sealed foil packs (rebuilt only when a pack's state changes; the CSS
    /// animations start with negative delays, so a rebuild continues them where they are).
    fn draw_packs(&mut self, ctx: &mut Ctx, s: &crate::save::GameSave, faces: &mut CardFaces, sw: f32) {
        if self.rewards.is_empty() {
            ctx.ui.set("packs", "");
            return;
        }
        let n = self.rewards.len();
        let urls: Vec<Option<String>> = self.rewards.iter().map(|&c| faces.url(Face::new(c, cards::level(s, c)))).collect();
        let state = |t: f32| if t <= PACK_SEALED + 1.0 { 's' } else if t < 0.0 { 'w' } else if t < PACK_GONE { 'o' } else { 'g' };
        let all = (0..n).all(|i| self.tear[i] >= PACK_REVEAL);
        let key: String = (0..n).map(|i| format!("{}{}", state(self.tear[i]), urls[i].is_some() as u8)).collect::<String>() + if all { "A" } else { "" } + if self.reward_t > 0.6 { "D" } else { "" };
        if key == self.pack_key {
            return;
        }
        self.pack_key = key;
        let w = ((sw - 100.0) / 3.2).clamp(90.0, 150.0);
        let mut packs = String::new();
        for i in 0..n {
            let c = self.rewards[i];
            let rare = cards::pack_rare(c);
            let t = self.tear[i];
            let sealed = t <= PACK_SEALED + 1.0;
            let img = urls[i].as_ref().map(|u| format!("<img src='{u}' alt=''>")).unwrap_or_default();
            // dealt in from above, one after another (only in the first moment)
            let deal = if self.reward_t < 0.6 { format!("--deal:{:.2}s;", 0.08 * i as f32 - self.reward_t) } else { "--deal:-1s;".into() };
            let tt = if sealed { 0.0 } else { t };
            let foil = if t < PACK_GONE {
                let art = format!(
                    "<div class='pk-art{}'><i class='pk-seam'></i><i class='pk-seam b'></i><div class='pk-logo'><em>{}</em><b>BRICK<br>BEASTS</b><small>{}</small></div><i class='pk-holo'></i></div>",
                    if rare { " rare" } else { "" },
                    if rare { "\u{2726}" } else { "\u{2605}" },
                    if rare { "RARE &middot; 1 CARD" } else { "1 CARD" }
                );
                format!("<div class='pk-p pk-l'>{art}</div><div class='pk-p pk-r'>{art}</div><div class='pk-p pk-s'>{art}</div>")
            } else {
                String::new()
            };
            packs += &format!(
                "<div class='pk{}{}' {} style='{deal}--tt:-{tt:.2}s;width:{w:.0}px;height:{:.0}px;--r0:{}deg;--pf:{:.1}px'><div class='pk-card'>{img}</div>{}{foil}</div>",
                if sealed { " sealed" } else { " torn" },
                if rare { " rare" } else { "" },
                if sealed { format!("data-action='pack:{i}'") } else { String::new() },
                w * 1.4,
                if i == 1 { 4 } else { -6 },
                w * 0.087,
                if !sealed && t < PACK_REVEAL + 0.9 { "<i class='pk-spark'></i>" } else { "" },
            );
        }
        let foot = if all {
            "<button class='pk-btn' data-action='pack_ok' style='background:#0055bf'>Awesome!</button>".to_string()
        } else {
            "<div class='pk-hint'>Tap a pack to tear it open!</div><button class='pk-btn' data-action='pack_all' style='background:#783cc8'>Reveal all</button>".to_string()
        };
        ctx.ui.set("packs", &format!("<div class='pk-shade'><div class='pk-box'><div class='pk-title'>{}</div><div class='pk-row'>{packs}</div>{foot}</div></div>", esc(&self.title)));
    }

    /// For `game_state`.
    pub fn describe(&self, save: &Save, faces: &CardFaces) -> Value {
        let s = &save.data;
        json!({
            "owned": cards::owned_count(s),
            "collection": (0..CARD_COUNT).filter(|&c| cards::owns(s, c)).map(|c| cards::card(c).id).collect::<Vec<_>>(),
            "deck": s.deck.iter().map(|&c| cards::card(c as usize).id).collect::<Vec<_>>(),
            "exp": s.exp,
            "levels": (0..CARD_COUNT).filter(|&c| cards::level(s, c) > 0).map(|c| (cards::card(c).id.to_string(), json!(cards::level(s, c)))).collect::<serde_json::Map<_, _>>(),
            "voxelmon": if s.voxelmon > 0 { Some(cards::card(s.voxelmon as usize - 1).id) } else { None },
            "deck_open": self.deck_open,
            "narrow": self.narrow,
            "pick": self.pick.map(|c| cards::card(c).id),
            "replacing": self.replacing.map(|c| cards::card(c).id),
            "adds": self.adds,
            "upgrades": self.upgrades,
            "upgrade": self.up.as_ref().map(|u| json!({"card": cards::card(u.card).id, "from": u.from, "to": u.to, "t": (u.t * 100.0).round() / 100.0})),
            "rewards": if self.rewards.is_empty() { Value::Null } else {
                json!({
                    "title": self.title,
                    "cards": self.rewards.iter().map(|&c| cards::card(c).id).collect::<Vec<_>>(),
                    "tear": self.tear[..self.rewards.len()].iter().map(|t| if *t <= PACK_SEALED + 1.0 { Value::Null } else { json!((t * 100.0).round() / 100.0) }).collect::<Vec<_>>(),
                    "revealed": self.tear[..self.rewards.len()].iter().filter(|t| **t >= PACK_REVEAL).count(),
                    "after": self.after.map(|d| format!("{d:?}")),
                })
            },
            "faces": {"drawn": faces.drawn, "urls": faces.ready(), "textures": faces.textures()},
        })
    }
}

/// 0..1, fixed per index (Kaigen `cu_hash`).
fn hash(mut i: u32) -> f32 {
    i = (i ^ 61) ^ (i >> 16);
    i = i.wrapping_mul(9);
    i ^= i >> 4;
    i = i.wrapping_mul(0x27d4eb2d);
    i ^= i >> 15;
    (i & 0xffff) as f32 / 65535.0
}

/// The card screens' styles (a style slot).
pub const STYLE: &str = "<style>
#ui .bh-deckbtn{position:fixed;right:62px;top:14px;margin:0;height:36px;padding:0 14px;border-radius:10px;background:rgba(0,85,191,.92);color:#fff;font-size:15px;font-weight:600;box-shadow:0 2px 6px rgba(0,0,0,.25);z-index:19;pointer-events:auto}
#ui .bh-deckbtn.narrow{right:14px;top:58px;width:56px;height:46px;padding:0;display:flex;flex-direction:column;align-items:center;justify-content:center;font-size:14px;line-height:1.1}#ui .bh-deckbtn.narrow small{font-size:12px;opacity:.85}
#ui .bc{position:relative;flex:none;border-radius:6%/4.3%;cursor:pointer;pointer-events:auto;transition:transform .12s,box-shadow .12s;filter:drop-shadow(0 3px 4px rgba(0,0,0,.3))}
#ui .bc img{display:block;width:100%;height:100%;-webkit-user-drag:none;user-select:none}
#ui .bc.unk{cursor:default}
#ui .bc.lv3{box-shadow:0 0 10px rgba(255,210,61,.6);border-radius:6%/4.3%}
#ui .bc.lv4{animation:bc-vox 2.4s ease-in-out infinite;border-radius:6%/4.3%}
@keyframes bc-vox{0%,100%{box-shadow:0 0 2px rgba(120,220,255,0)}50%{box-shadow:0 0 16px 2px rgba(120,220,255,.85)}}
#ui [data-slot^=deck_]{pointer-events:none}
.dk-head{position:fixed;left:0;right:0;top:0;z-index:12;box-sizing:border-box;display:flex;align-items:center;gap:12px;padding:10px 12px;padding-left:max(12px,env(safe-area-inset-left));padding-right:max(12px,env(safe-area-inset-right));background:#cec7b9;color:#1b2a34;box-shadow:0 2px 8px rgba(0,0,0,.18);pointer-events:auto}
.dk-title{flex:1;min-width:0}.dk-title b{font-size:24px;font-weight:800}
.dk-counts{display:flex;flex-wrap:wrap;gap:0 12px;font-size:14px}.dk-ok{color:#1e823c}.dk-bad{color:#c83c28}.dk-exp{color:#783cc8}
#ui .dk-close{margin:0;height:42px;border-radius:21px;padding:0 16px 0 12px;background:#3c4048;color:#fff;font-size:15px;font-weight:700;display:flex;gap:6px;align-items:center}
#ui .dk-close b{font-size:24px;line-height:1}
.dk-coll{position:fixed;left:0;z-index:10;overflow-y:auto;overscroll-behavior:contain;box-sizing:border-box;padding:10px 12px 0;background:#cec7b9;color:#1b2a34;pointer-events:auto;-webkit-overflow-scrolling:touch}
.dk-coll::before{content:'';position:fixed;inset:0;background:#cec7b9;z-index:-1}
.dk-cap{font-size:14px;margin-bottom:8px}
.dk-grid{display:flex;flex-wrap:wrap;gap:8px;padding:16px 0 0}
.dk-pad{height:12px}
.dk-strip{position:fixed;left:0;right:0;bottom:0;z-index:12;box-sizing:border-box;display:flex;flex-direction:column;align-items:center;gap:6px;padding:8px 20px 12px;background:#282c34;box-shadow:0 -2px 10px rgba(0,0,0,.3);pointer-events:auto}
.dk-repl{flex-direction:column;align-items:stretch}.dk-repl p{margin:0;font-size:14px}
.dk-rgrid{display:flex;flex-wrap:wrap;gap:6px;justify-content:center}.dk-rgrid .bc{cursor:pointer}
.dk-row{display:flex;gap:6px}.dk-empty{border-radius:6px;background:#464a52}
.dk-slot.lit .bc{transform:translateY(-6px);box-shadow:0 0 12px 4px #ffd23d}
.dk-side{position:fixed;right:12px;z-index:11;box-sizing:border-box;overflow-y:auto;display:flex;flex-direction:column;align-items:center;gap:8px;padding:12px;background:#fff;border-radius:12px;color:#1b2a34;pointer-events:auto}
.dk-side p{margin:0;font-size:14px}
#dk-big{cursor:default;filter:drop-shadow(0 4px 8px rgba(0,0,0,.35))}
.dk-text{font-size:14px;width:100%}.dk-stage{font-size:13px;color:#5a5e64;width:100%}
#ui .dk-btn{margin:0;width:100%;padding:8px 10px;border-radius:9px;color:#fff;font-size:14px;font-weight:700;display:flex;flex-direction:column;align-items:center;gap:1px}
#ui .dk-btn small{font-size:11px;font-weight:500;opacity:.9}#ui .dk-btn[disabled]{background:#969aa0!important;cursor:default}
.dk-sheet{position:fixed;left:0;right:0;bottom:0;z-index:20;box-sizing:border-box;display:flex;gap:12px;padding:16px 14px;padding-bottom:max(16px,env(safe-area-inset-bottom));background:#fff;border-radius:18px 18px 0 0;box-shadow:0 -4px 20px rgba(0,0,0,.35);color:#1b2a34;pointer-events:auto;max-height:60vh;overflow-y:auto}
.dk-acts{flex:1;min-width:0;display:flex;flex-direction:column;gap:8px}
.dk-sheethead{display:flex;align-items:center}.dk-sheethead b{flex:1;font-size:18px;font-weight:800}
#ui .dk-x{margin:0;width:34px;height:34px;border-radius:17px;background:#e2e6ea;color:#1b2a34;font-size:20px;padding:0}
#ui [data-slot=packs]{position:fixed;inset:0;z-index:40;pointer-events:none}#ui [data-slot=packs]:empty{display:none}
.pk-shade{position:fixed;inset:0;display:flex;align-items:center;justify-content:center;padding:12px;background:rgba(0,0,0,.55);pointer-events:auto}
.pk-box{display:flex;flex-direction:column;align-items:center;gap:14px;padding:22px;background:#fafaf6;border-radius:14px;box-shadow:0 10px 30px rgba(0,0,0,.4);color:#1b2a34;max-width:calc(100vw - 24px);box-sizing:border-box}
.pk-title{font-size:20px;font-weight:800;text-align:center;max-width:520px}
.pk-row{display:flex;gap:14px;padding:8px 0}
.pk-hint{font-size:14px;color:#5a5e64}
#ui .pk-btn{margin:0;padding:9px 18px;border-radius:9px;color:#fff;font-size:17px;font-weight:600}
.pk{position:relative;animation:pk-deal .35s ease-out both;animation-delay:var(--deal)}
.pk.sealed{cursor:pointer;pointer-events:auto}.pk.sealed:hover .pk-p{filter:brightness(1.12)}
@keyframes pk-deal{from{opacity:0;transform:translateY(-40%) rotate(var(--r0))}to{opacity:1;transform:none}}
.pk-card{position:absolute;inset:0;border-radius:6%/4.3%}.pk-card img{width:100%;height:100%;display:block}
.pk.torn .pk-card{animation:pk-pop .55s both;animation-delay:calc(var(--tt) + .42s)}
.pk.torn.rare .pk-card{box-shadow:0 0 18px 2px rgba(255,210,61,.9)}
@keyframes pk-pop{0%{transform:scale(.85)}60%{transform:scale(1.1) rotate(-2deg)}100%{transform:none}}
.pk-p{position:absolute;left:-1.5%;top:-3%;width:103%;height:106%;z-index:2}
.pk-l{clip-path:polygon(0 0,52% 0,46% 25%,54% 50%,46% 75%,52% 100%,0 100%)}
.pk-r{clip-path:polygon(52% 0,100% 0,100% 100%,52% 100%,46% 75%,54% 50%,46% 25%)}
.pk-s{z-index:3;clip-path:polygon(0 0,100% 0,100% 11%,75% 13.5%,50% 11%,25% 13.5%,0 11.5%)}
.pk.torn .pk-s{animation:pk-strip .35s ease-in forwards;animation-delay:var(--tt)}
.pk.torn .pk-l{animation:pk-left .55s ease-in forwards;animation-delay:calc(var(--tt) + .25s)}
.pk.torn .pk-r{animation:pk-right .55s ease-in forwards;animation-delay:calc(var(--tt) + .25s)}
@keyframes pk-strip{to{transform:translate(35%,-50%) rotate(-24deg);opacity:0}}
@keyframes pk-left{to{transform:translateX(-75%) rotate(-16deg);opacity:0}}
@keyframes pk-right{to{transform:translateX(75%) rotate(16deg);opacity:0}}
.pk-art{position:absolute;inset:0;overflow:hidden;border-radius:3.5%/2.5%;display:flex;align-items:center;justify-content:center;background:linear-gradient(115deg,#b9c0cc 0%,#f6f8fb 30%,#98a2b3 50%,#eef1f6 100%)}
.pk-art.rare{background:linear-gradient(115deg,#c99a2e 0%,#fff2b8 30%,#b8862a 50%,#ffe68a 100%)}
.pk-seam{position:absolute;left:0;right:0;top:0;height:9%;background:repeating-linear-gradient(90deg,rgba(0,0,0,.18) 0 2px,rgba(255,255,255,.35) 2px 4px)}.pk-seam.b{top:auto;bottom:0}
.pk-logo{display:flex;flex-direction:column;align-items:center;gap:.1em;transform:rotate(-8deg);text-align:center;font-size:var(--pf,14px)}
.pk-logo em{font-style:normal;color:#ffd23d;font-size:2.4em;line-height:1;text-shadow:0 1px 0 #a87a00}
.pk-logo b{color:#c91a09;font-size:1.5em;font-weight:800;line-height:1.05;text-shadow:0 1px 0 #fff}.pk-art.rare .pk-logo b{color:#8a4b00}
.pk-logo small{color:#1b2a34;font-size:.85em;font-weight:700}
.pk-holo{position:absolute;top:0;bottom:0;width:90%;left:-100%;background:linear-gradient(115deg,rgba(255,80,160,0),rgba(255,80,160,.22) 33%,rgba(80,200,255,.25) 66%,rgba(120,255,140,0));animation:pk-holo 2.4s linear infinite}
@keyframes pk-holo{from{left:-110%}to{left:110%}}
.pk-spark{position:absolute;left:50%;top:50%;width:130%;aspect-ratio:1;z-index:4;border-radius:50%;background:radial-gradient(closest-side,rgba(255,255,255,.95),rgba(255,230,120,.55) 40%,rgba(255,230,120,0) 70%);transform:translate(-50%,-50%) scale(0);opacity:0;animation:pk-spark .9s ease-out both;animation-delay:calc(var(--tt) + .42s);pointer-events:none}
@keyframes pk-spark{0%{transform:translate(-50%,-50%) scale(.6);opacity:1}100%{transform:translate(-50%,-50%) scale(1.4);opacity:0}}
#ui [data-slot=upgrade]{position:fixed;inset:0;z-index:45;pointer-events:none}#ui [data-slot=upgrade]:empty{display:none}
.up{position:fixed;inset:0;overflow:hidden;pointer-events:auto;background:radial-gradient(circle at 50% 42%,rgba(42,53,70,.9),rgba(5,7,11,.97));animation:up-in .3s both;animation-delay:var(--t)}
@keyframes up-in{from{opacity:0}}
.up.vox{animation:up-in .3s both,up-open .45s forwards;animation-delay:var(--t),calc(var(--t) + 1.25s)}
@keyframes up-open{to{background:radial-gradient(circle at 50% 42%,rgba(42,53,70,.15),rgba(5,7,11,.15))}}
.up-rays{position:absolute;left:50%;top:42%;animation:up-spin 20s linear infinite}
.up-rays i{position:absolute;left:calc(var(--cw) * -.16);top:-75vmax;width:calc(var(--cw) * .32);height:150vmax;transform:rotate(var(--a));background:linear-gradient(transparent,color-mix(in srgb,var(--f) 24%,transparent),transparent)}
@keyframes up-spin{to{transform:rotate(360deg)}}
.up-sparks{position:absolute;left:50%;top:42%}
.up-sparks b{position:absolute;width:8px;height:8px;margin:-4px;background:var(--c);box-shadow:0 0 8px var(--f);opacity:0;animation:up-spark .9s ease-in both;animation-delay:calc(var(--t) + .25s + var(--d))}
@keyframes up-spark{0%{opacity:0;transform:rotate(var(--a)) translateX(calc(var(--cw) * 1.8)) rotate(45deg)}20%{opacity:1}100%{opacity:1;transform:rotate(calc(var(--a) + 2.7rad)) translateX(calc(var(--cw) * .2)) rotate(45deg)}}
.up-card{position:absolute;left:50%;top:42%;width:var(--cw);height:calc(var(--cw) * 1.4);margin:calc(var(--cw) * -.7) 0 0 calc(var(--cw) * -.5);animation:up-cardin .35s ease-out both,up-flip 1.6s both;animation-delay:var(--t),var(--t)}
.up-card img{position:absolute;inset:0;width:100%;height:100%}
.up-card>div{position:absolute;inset:0;border-radius:6%/4.3%;box-shadow:0 0 calc(var(--cw) * .3) var(--f)}
.up-new{opacity:0;animation:up-show .01s forwards;animation-delay:calc(var(--t) + 1.3s)}
.up-old{animation:up-hide .01s forwards;animation-delay:calc(var(--t) + 1.3s)}
@keyframes up-show{to{opacity:1}}@keyframes up-hide{to{opacity:0}}
@keyframes up-cardin{from{opacity:0;scale:.6}}
@keyframes up-flip{0%{transform:none}25%{transform:rotate(1.5deg)}35%{transform:rotate(-1.5deg)}45%{transform:rotate(1.5deg)}62.5%{transform:scaleX(1)}81%{transform:scaleX(.02)}100%{transform:scaleX(1) scale(1.08)}}
.up.vox .up-card{animation:up-cardin .35s ease-out both,up-shake .12s linear infinite,up-gone .01s forwards;animation-delay:var(--t),calc(var(--t) + .3s),calc(var(--t) + 1.25s)}
.up.vox .up-new,.up.vox .up-old{animation:none}.up.vox .up-new{opacity:0}
.up.vox .up-card>div{box-shadow:0 0 calc(var(--cw) * .35) #b07aff}
@keyframes up-shake{0%,100%{transform:rotate(-3deg) scale(1.06)}50%{transform:rotate(3deg) scale(1.06)}}
@keyframes up-gone{to{visibility:hidden}}
.up-cubes{position:absolute;left:50%;top:45%}
.up-cubes s{position:absolute;width:var(--s);height:var(--s);margin:calc(var(--s) * -.5);background:linear-gradient(135deg,color-mix(in srgb,var(--c),#fff 35%),var(--c));border:1px solid rgba(0,0,0,.25);border-radius:12%;opacity:0;animation:up-cube 1.1s ease-in both;animation-delay:calc(var(--t) + 1.25s)}
@keyframes up-cube{0%{opacity:1;transform:translate(var(--x),var(--y))}100%{opacity:0;transform:translate(calc(var(--x) + var(--vx)),calc(var(--y) + var(--vy) + var(--g))) rotate(var(--r)) scale(.6)}}
.up-flash{position:absolute;inset:0;background:#fff;opacity:0;animation:up-flash .4s both;animation-delay:calc(var(--t) + 1.25s);pointer-events:none}
@keyframes up-flash{12%{opacity:.55}100%{opacity:0}}
.up-words{position:absolute;left:50%;top:calc(42% + var(--cw) * .7 + 18px);width:min(560px,calc(100vw - 24px));transform:translateX(-50%);display:flex;flex-direction:column;align-items:center;gap:6px;text-align:center;opacity:0;animation:up-pop .35s forwards;animation-delay:calc(var(--t) + 1.5s)}
.up.vox .up-words{top:6%;animation-delay:calc(var(--t) + 1.85s)}
.up-words b{font-size:clamp(26px,7.5vw,44px);font-weight:800;color:color-mix(in srgb,var(--f),#fff 25%);text-shadow:0 2px 6px rgba(0,0,0,.6)}
.up.vox .up-words b{color:#d9b8ff}
.up-words span{font-size:16px;font-weight:600;color:rgba(255,255,255,.92);text-shadow:0 1px 4px rgba(0,0,0,.7)}
@keyframes up-pop{0%{opacity:0;scale:.6}70%{opacity:1;scale:1.05}100%{opacity:1;scale:1}}
#ui .up-ok{position:absolute;left:50%;bottom:calc(7% + env(safe-area-inset-bottom));transform:translateX(-50%);margin:0;padding:9px 20px;border-radius:9px;background:#0055bf;color:#fff;font-size:20px;opacity:0;animation:up-pop .3s forwards;animation-delay:calc(var(--t) + 2s)}
#ui .up.vox .up-ok{background:#7a3fd6;animation-delay:calc(var(--t) + 2.7s)}
</style>";
