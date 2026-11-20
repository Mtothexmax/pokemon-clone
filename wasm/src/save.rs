//! The save game (Kaigen `st_save.c`): one serde struct in localStorage (namespace "brickhollow", slot
//! "save"), read once when the game starts, written half a second after something changed (bursts of
//! changes coalesce).
//!
//! Hooks for later parts: add fields to [`GameSave`] (every field has a serde default, so older saves
//! load with the new fields at their default) and call [`Save::mark`] after changing one. The quest
//! flags of the Kaigen save are already here (`has_deck`, `quest_part`, `has_suit`, `has_part`,
//! `rival_wins`, `voxelmon`, `gate_open`, ...): the dialogues ([`crate::dialog::dialog_for`]) and the
//! goal pill ([`crate::talk::goal_text`]) read them.

use engine::prelude::*;
use serde::{Deserialize, Serialize};

const SLOT: &str = "save";

/// Graphics choice of the menu (Kaigen `GfxChoice`).
pub const GFX_AUTO: u8 = 0;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSave {
    /// Times the village was entered (the first slice's counter).
    pub plays: u32,
    // ---- progress (Kaigen GameSave, in the same order) ----
    pub golden_mask: u32,
    pub studs: u32,
    pub duck_feeds: u32,
    pub chests: u32,
    pub fireflies: u32,
    pub goals: u32,
    pub seek_wins: u32,
    pub legendary: bool,
    /// Hours 0..24: the day goes on where you left it.
    pub time_of_day: f32,
    // ---- cards and Voxelmon ----
    pub has_deck: bool,
    pub collection: u64,
    pub deck: Vec<u8>,
    pub card_level: Vec<u8>,
    pub exp: u32,
    pub wins: u32,
    pub losses: u32,
    pub rival_wins: u32,
    /// The companion: CardId + 1, 0 none.
    pub voxelmon: i32,
    pub gate_open: bool,
    pub vox_wins: u32,
    pub met_rival: bool,
    // ---- the menu ----
    /// 0 Auto, 1 High, 2 Medium, 3 Low.
    pub gfx: u8,
    /// The step Auto reached + 1 (0: not chosen yet).
    pub auto_step: u8,
    pub voices_off: bool,
    /// Music volume step: 0 off, 1 quiet, 2 normal (the Kaigen level), 3 loud.
    pub music_level: u8,
    // ---- the first deck's quest and the pond cave ----
    pub quest_part: bool,
    pub has_suit: bool,
    pub has_part: bool,
    pub dive_found: u32,
    // ---- Pip ----
    pub seek_played: u32,
    pub pip_wins: u32,
    pub tag_wins: u32,
    pub bench_sits: u32,
    /// The intro thought was shown (the bedroom start, rooms part).
    pub intro_seen: bool,
}

impl Default for GameSave {
    fn default() -> Self {
        Self {
            plays: 0,
            golden_mask: 0,
            studs: 0,
            duck_feeds: 0,
            chests: 0,
            fireflies: 0,
            goals: 0,
            seek_wins: 0,
            legendary: false,
            time_of_day: crate::START_HOUR,
            has_deck: false,
            collection: 0,
            deck: vec![],
            card_level: vec![],
            exp: 0,
            wins: 0,
            losses: 0,
            rival_wins: 0,
            voxelmon: 0,
            gate_open: false,
            vox_wins: 0,
            met_rival: false,
            gfx: GFX_AUTO,
            auto_step: 0,
            voices_off: false,
            music_level: 2,
            quest_part: false,
            has_suit: false,
            has_part: false,
            dive_found: 0,
            seek_played: 0,
            pip_wins: 0,
            tag_wins: 0,
            bench_sits: 0,
            intro_seen: false,
        }
    }
}

impl GameSave {
    pub fn golden_count(&self) -> u32 {
        self.golden_mask.count_ones()
    }
}

#[derive(Default)]
pub struct Save {
    pub data: GameSave,
    pub loaded: bool,
    dirty: bool,
    wait: f32,
    /// Writes done (for tests).
    pub writes: u32,
    /// Never write (a trial run, Kaigen F3).
    pub no_write: bool,
}

impl Save {
    /// Reads the save (once).
    pub fn load(&mut self, ctx: &mut Ctx) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        ctx.save.set_namespace("brickhollow", 1);
        match ctx.save.read::<GameSave>(SLOT) {
            Some(g) => {
                self.data = g;
                log_info!("save: loaded (studs {}, golden {})", self.data.studs, self.data.golden_count());
            }
            None => log_info!("save: new game"),
        }
    }

    /// Something changed: written half a second later.
    pub fn mark(&mut self) {
        self.dirty = true;
        self.wait = 0.5;
    }

    /// Every frame: writes a dirty save once it has rested.
    pub fn update(&mut self, ctx: &mut Ctx, dt: f32) {
        if !self.dirty || self.no_write {
            return;
        }
        self.wait -= dt;
        if self.wait > 0.0 {
            return;
        }
        self.flush(ctx);
    }

    /// Writes now (leaving the screen, tests).
    pub fn flush(&mut self, ctx: &mut Ctx) {
        if self.no_write {
            return;
        }
        self.dirty = false;
        match ctx.save.write(SLOT, &self.data) {
            Ok(_) => self.writes += 1,
            Err(e) => log_warn!("save: write failed: {e:?}"),
        }
    }

    /// A new game (debug command `save_reset`).
    pub fn reset(&mut self, ctx: &mut Ctx) {
        self.data = GameSave::default();
        self.flush(ctx);
    }
}
