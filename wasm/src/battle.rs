//! The card battle's rules (Kaigen `st_battle.c`, the three.js `battle.svelte.ts`): 3 locations out of 12, 6 turns,
//! energy = turn, win 2 of 3 locations (a tie: the higher total). You plan in secret, then both sides reveal
//! (whoever leads more locations first; a tie: a coin), On Reveal / Activate / End of Turn abilities fire with
//! short pauses (a step machine driven by `update(dt)`), the AI plans like the three.js one. SNAP doubles the
//! stakes, the final turn doubles them again.
//!
//! Pure game logic, no engine: the 3D table (`arena.rs`) reads the cards' timers (`fx_t`, `pulse_t`, `die_t`,
//! `draw_t`) and the sound counters (`sfx_*`) like the Kaigen UI did.

use crate::cards::{self, Kind as CardKind};

pub const MAX_CARDS: usize = 48;
pub const HAND_MAX: usize = 7;
pub const SIDE_P: usize = 0;
pub const SIDE_AI: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum BattleKind {
    Wild,
    Rival,
    Practice,
    Pip,
}

impl BattleKind {
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "rival" | "dex" => Some(Self::Rival),
            "practice" | "lumi" => Some(Self::Practice),
            "pip" => Some(Self::Pip),
            "wild" => Some(Self::Wild),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    Plan,
    Reveal,
    Over,
}

/// The result (Kaigen `BattleResult`; `code()` is what `cards::battle_exp` takes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Outcome {
    Win,
    Lose,
    Draw,
    Retreat,
    None,
}

impl Outcome {
    pub fn code(self) -> i32 {
        match self {
            Outcome::Win => 0,
            Outcome::Lose => 1,
            Outcome::Draw => 2,
            _ => 3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BCard {
    pub id: usize,
    pub owner: usize,
    /// -1: hand or deck
    pub zone: i32,
    /// permanent power changes from abilities
    pub modp: i32,
    pub revealed: bool,
    pub staged: bool,
    pub activated: bool,
    pub alive: bool,
    /// play order this turn
    pub order: i32,
    /// an Activate chosen this turn (-1 none)
    pub act_order: i32,
    /// when it entered the hand (the hand keeps this order)
    pub hseq: u32,
    /// the last power change and the seconds since it (the table shows +N / -N, a projectile flies from `fx_src`)
    pub fx_n: i32,
    pub fx_t: f32,
    pub fx_src: i32,
    /// seconds since it was revealed or activated
    pub pulse_t: f32,
    /// seconds since it was destroyed
    pub die_t: f32,
    /// seconds since it came into the hand (it flies out of the deck)
    pub draw_t: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StepKind {
    Reveal,
    Act,
}

#[derive(Clone, Copy, Debug)]
struct Step {
    kind: StepKind,
    card: usize,
    at: i32,
}

#[derive(Clone, Debug)]
pub struct Battle {
    #[allow(dead_code)]
    pub kind: BattleKind,
    pub opponent: String,
    pub c: Vec<BCard>,
    /// the three locations (indices into `cards::ZONES`) and whether they are revealed
    pub zone: [usize; 3],
    pub zone_rev: [bool; 3],
    pub hand: [Vec<usize>; 2],
    pub deck: [Vec<usize>; 2],
    pub turn: i32,
    pub max_turns: i32,
    pub energy: i32,
    pub energy_left: i32,
    pub phase: Phase,
    pub result: Outcome,
    pub stakes: i32,
    pub p_snapped: bool,
    pub ai_snapped: bool,
    /// who reveals first this turn: decided when the turn starts
    pub first: usize,
    /// seconds since the reveal began (the "X reveals first" banner)
    pub first_t: f32,
    play_counter: i32,
    hseq: u32,
    pub seed: u32,
    steps: Vec<Step>,
    pub step_i: usize,
    reveal_repeat: i32,
    /// 0 reveals, 1 end of turn, 2 next turn
    stage: i32,
    et_side: usize,
    et_zone: usize,
    et_idx: usize,
    pub wait: f32,
    /// newest first
    pub log: Vec<String>,
    pub sfx_hits: u32,
    pub sfx_buffs: u32,
    pub sfx_kills: u32,
    pub sfx_reveals: u32,
}

pub fn other(s: usize) -> usize {
    1 - s
}

impl Battle {
    fn rand_u(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        self.seed >> 8
    }
    fn rand(&mut self) -> f32 {
        self.rand_u() as f32 / 16777216.0
    }

    pub fn say(&mut self, msg: impl Into<String>) {
        self.log.insert(0, msg.into());
        self.log.truncate(6);
    }

    fn name(&self, ci: usize) -> &'static str {
        cards::card(self.c[ci].id).name
    }

    /// Revealed cards of a side at a zone.
    pub fn cards_at(&self, zone: i32, side: usize) -> Vec<usize> {
        if zone < 0 {
            return vec![];
        }
        (0..self.c.len()).filter(|&i| {
            let c = &self.c[i];
            c.alive && c.zone == zone && c.owner == side && c.revealed
        })
        .collect()
    }

    /// Every card on a side of a zone, staged ones too (for the capacity).
    pub fn count_at(&self, zone: usize, side: usize) -> usize {
        self.c.iter().filter(|c| c.alive && c.zone == zone as i32 && c.owner == side).count()
    }

    pub fn capacity(&self, zone: usize) -> usize {
        if self.zone_rev[zone] && cards::ZONES[self.zone[zone]].id == "ledge" {
            2
        } else {
            4
        }
    }

    fn pick(&mut self, arr: &[usize]) -> Option<usize> {
        if arr.is_empty() {
            None
        } else {
            let r = self.rand_u() as usize;
            Some(arr[r % arr.len()])
        }
    }

    fn buff(&mut self, target: usize, n: i32, src: usize) {
        let c = &mut self.c[target];
        c.modp += n;
        c.fx_n = n;
        c.fx_t = 0.0;
        c.fx_src = src as i32;
        if n > 0 {
            self.sfx_buffs += 1;
        } else {
            self.sfx_hits += 1;
        }
        let msg = format!("{}: {} {}{} Power", self.name(src), self.name(target), if n > 0 { "+" } else { "" }, n);
        self.say(msg);
    }

    fn destroy(&mut self, target: usize, src: usize) {
        let c = &mut self.c[target];
        c.alive = false;
        c.die_t = 0.0;
        c.fx_src = src as i32;
        self.sfx_kills += 1;
        let msg = format!("{} destroyed {}!", self.name(src), self.name(target));
        self.say(msg);
    }

    fn draw(&mut self, side: usize, silent: bool) {
        if self.deck[side].is_empty() || self.hand[side].len() >= HAND_MAX {
            return;
        }
        let ci = self.deck[side].remove(0);
        self.c[ci].hseq = self.hseq;
        self.hseq += 1;
        self.c[ci].draw_t = 0.0;
        self.hand[side].push(ci);
        if !silent && side == SIDE_P {
            let msg = format!("You drew {}.", self.name(ci));
            self.say(msg);
        }
    }

    fn new_card(&mut self, id: usize, owner: usize) -> usize {
        self.c.push(BCard {
            id,
            owner,
            zone: -1,
            modp: 0,
            revealed: false,
            staged: false,
            activated: false,
            alive: true,
            order: 0,
            act_order: -1,
            hseq: 0,
            fx_n: 0,
            fx_t: 99.0,
            fx_src: -1,
            pulse_t: 99.0,
            die_t: 99.0,
            draw_t: 99.0,
        });
        self.c.len() - 1
    }

    fn add_to_hand(&mut self, side: usize, id: usize) {
        if self.hand[side].len() >= HAND_MAX || self.c.len() >= MAX_CARDS {
            return;
        }
        let ci = self.new_card(id, side);
        self.c[ci].hseq = self.hseq;
        self.hseq += 1;
        self.c[ci].draw_t = 0.0;
        self.hand[side].push(ci);
        let who = if side == SIDE_P { "You".to_string() } else { self.opponent.clone() };
        self.say(format!("{who} got a {}.", cards::card(id).name));
    }

    fn to_opening_hand(&mut self, ci: usize) {
        let s = self.c[ci].owner;
        if let Some(i) = self.deck[s].iter().position(|&d| d == ci) {
            self.deck[s].remove(i);
            self.deck[s].insert(0, ci);
        }
    }

    // ---- power ----

    fn self_bonus(&self, ci: usize) -> i32 {
        let c = &self.c[ci];
        let n = self.cards_at(c.zone, c.owner).len();
        match c.id {
            cards::ZEPHYRROW => {
                if n == 1 {
                    4
                } else {
                    0
                }
            }
            cards::BRAMBLEWOLF => n as i32 - 1,
            cards::CRAGOLIATH => {
                if n >= self.capacity(c.zone as usize) {
                    3
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// The power card `src` (revealed, Ongoing) gives `t`.
    fn aura(&self, src: usize, t: usize) -> i32 {
        let (s, c) = (&self.c[src], &self.c[t]);
        match s.id {
            cards::MOSSCUB => (c.owner == s.owner && c.zone == s.zone && t != src) as i32,
            cards::GLOOMOTH => -((c.owner != s.owner && c.zone == s.zone) as i32),
            cards::CORALYTH => (c.owner == s.owner && c.zone != s.zone) as i32,
            cards::ABYSSQUID => 2 * (c.owner == s.owner && c.zone == s.zone && t != src) as i32,
            _ => 0,
        }
    }

    pub fn zone_bonus(&self, zone: usize, ci: usize) -> i32 {
        let d = cards::card(self.c[ci].id);
        match cards::ZONES[self.zone[zone]].id {
            "pond" => 2 * (d.kind == CardKind::Ongoing) as i32,
            "grove" => 2 * (d.kind == CardKind::None) as i32,
            "barn" => -1,
            "gym" => 1,
            "toybox" => 2 * (d.cost <= 2) as i32,
            "workshop" => 2 * (d.kind == CardKind::Activate) as i32,
            _ => 0,
        }
    }

    pub fn power(&self, ci: usize) -> i32 {
        let c = &self.c[ci];
        let mut p = cards::card(c.id).power + c.modp;
        if c.zone < 0 || !c.revealed {
            return p;
        }
        p += self.self_bonus(ci);
        if self.zone_rev[c.zone as usize] {
            p += self.zone_bonus(c.zone as usize, ci);
        }
        for i in 0..self.c.len() {
            let o = &self.c[i];
            if o.alive && o.revealed && o.zone >= 0 {
                p += self.aura(i, ci);
            }
        }
        p
    }

    pub fn zone_power(&self, zone: usize, side: usize) -> i32 {
        self.cards_at(zone as i32, side).iter().map(|&i| self.power(i)).sum()
    }

    /// The side winning a zone, or None on a tie.
    pub fn zone_winner(&self, zone: usize) -> Option<usize> {
        let (a, d) = (self.zone_power(zone, SIDE_P), self.zone_power(zone, SIDE_AI));
        if a > d {
            Some(SIDE_P)
        } else if d > a {
            Some(SIDE_AI)
        } else {
            None
        }
    }

    pub fn winning_zones(&self, side: usize) -> usize {
        (0..3).filter(|&z| self.zone_winner(z) == Some(side)).count()
    }

    // ---- abilities ----

    fn enemies_here(&self, ci: usize) -> Vec<usize> {
        self.cards_at(self.c[ci].zone, other(self.c[ci].owner))
    }
    fn friends_here(&self, ci: usize) -> Vec<usize> {
        self.cards_at(self.c[ci].zone, self.c[ci].owner).into_iter().filter(|&i| i != ci).collect()
    }

    fn on_reveal(&mut self, ci: usize) {
        let (id, owner, zone) = (self.c[ci].id, self.c[ci].owner, self.c[ci].zone);
        match id {
            cards::FLICKER => {
                for t in self.friends_here(ci) {
                    self.buff(t, 1, ci);
                }
            }
            cards::DRIZZLET => self.draw(owner, false),
            cards::ZAPLING => {
                let e = self.enemies_here(ci);
                if let Some(t) = self.pick(&e) {
                    self.buff(t, -1, ci);
                }
            }
            cards::VOLTAIL => {
                let e = self.enemies_here(ci);
                if let Some(t) = self.pick(&e) {
                    self.buff(t, -2, ci);
                }
            }
            cards::CINDERPUP => {
                if !self.enemies_here(ci).is_empty() {
                    self.buff(ci, 3, ci);
                }
            }
            cards::TIDALOTL => {
                for i in 0..self.c.len() {
                    let c = &self.c[i];
                    if c.alive && c.revealed && c.owner == owner && c.zone >= 0 && c.zone != zone {
                        self.buff(i, 1, ci);
                    }
                }
            }
            cards::FUNGOO => {
                for t in self.enemies_here(ci) {
                    self.buff(t, -1, ci);
                }
            }
            cards::GLIMMERLING => self.add_to_hand(owner, cards::BRICKLING),
            cards::THUNDRAKE => {
                let mut t: Option<usize> = None;
                for i in self.enemies_here(ci) {
                    if t.map_or(true, |t| self.power(i) > self.power(t)) {
                        t = Some(i);
                    }
                }
                if let Some(t) = t {
                    self.buff(t, -3, ci);
                }
            }
            cards::EMBERHORN => {
                for t in self.friends_here(ci) {
                    self.buff(t, -1, ci);
                }
            }
            cards::PHANTOMULE => {
                let mut t: Option<usize> = None;
                for i in self.enemies_here(ci) {
                    if t.map_or(true, |t| self.power(i) < self.power(t)) {
                        t = Some(i);
                    }
                }
                if let Some(t) = t {
                    self.destroy(t, ci);
                }
            }
            cards::GLACIOMOTH => {
                for t in self.enemies_here(ci) {
                    self.buff(t, -2, ci);
                }
            }
            cards::LUMIFOX => {
                for t in self.friends_here(ci) {
                    self.buff(t, 2, ci);
                }
            }
            cards::STORMAW => {
                for i in 0..self.c.len() {
                    let c = &self.c[i];
                    if c.alive && c.revealed && c.zone >= 0 && c.owner != owner {
                        self.buff(i, -1, ci);
                    }
                }
            }
            _ => {}
        }
    }

    fn activate_fx(&mut self, ci: usize) {
        match self.c[ci].id {
            cards::GUSTLING => {
                for t in self.friends_here(ci) {
                    self.buff(t, 1, ci);
                }
            }
            cards::SPARKMOLE => self.draw(self.c[ci].owner, false),
            cards::BOOMSHROOM => {
                for t in self.enemies_here(ci) {
                    self.buff(t, -2, ci);
                }
            }
            _ => {}
        }
    }

    fn end_turn_fx(&mut self, ci: usize) {
        match self.c[ci].id {
            cards::SPROUTLE => self.buff(ci, 1, ci),
            cards::TICKBOT => {
                let f = self.friends_here(ci);
                if let Some(t) = self.pick(&f) {
                    self.buff(t, 1, ci);
                }
            }
            cards::CLOCKCRAB => self.buff(ci, 2, ci),
            _ => {}
        }
    }

    fn has_end_turn(id: usize) -> bool {
        matches!(id, cards::SPROUTLE | cards::TICKBOT | cards::CLOCKCRAB)
    }
    pub fn has_activate(id: usize) -> bool {
        matches!(id, cards::GUSTLING | cards::SPARKMOLE | cards::BOOMSHROOM)
    }

    fn reveal_times(&self, ci: usize) -> i32 {
        let c = &self.c[ci];
        let z = c.zone as usize;
        let open = self.zone_rev[z];
        let zid = cards::ZONES[self.zone[z]].id;
        if open && zid == "library" {
            return 0;
        }
        let mut n = 1;
        if open && zid == "stage" {
            n += 1;
        }
        for i in self.cards_at(c.zone, c.owner) {
            if i != ci && self.c[i].id == cards::ECHOWL {
                n += 1;
            }
        }
        n
    }

    pub fn can_activate(&self, ci: usize) -> bool {
        let c = &self.c[ci];
        Self::has_activate(c.id) && c.alive && c.revealed && !c.activated && c.act_order < 0 && c.zone >= 0 && (c.owner != SIDE_P || self.phase == Phase::Plan)
    }

    pub fn activate(&mut self, ci: usize) -> bool {
        if !self.can_activate(ci) {
            return false;
        }
        self.c[ci].act_order = self.play_counter;
        self.play_counter += 1;
        self.c[ci].pulse_t = 0.0;
        true
    }

    pub fn cancel_activate(&mut self, ci: usize) {
        if self.phase == Phase::Plan && self.c[ci].act_order >= 0 {
            self.c[ci].act_order = -1;
        }
    }

    pub fn zone_open(&self, zone: usize, side: usize) -> bool {
        if self.count_at(zone, side) >= self.capacity(zone) {
            return false;
        }
        !(self.zone_rev[zone] && cards::ZONES[self.zone[zone]].id == "tower" && self.turn >= self.max_turns)
    }

    pub fn can_play(&self, ci: usize, zone: usize) -> bool {
        self.phase == Phase::Plan && zone < 3 && cards::card(self.c[ci].id).cost <= self.energy_left && self.zone_open(zone, SIDE_P)
    }

    pub fn playable(&self, ci: usize) -> bool {
        self.phase == Phase::Plan && cards::card(self.c[ci].id).cost <= self.energy_left
    }

    fn remove_from_hand(&mut self, side: usize, ci: usize) {
        self.hand[side].retain(|&h| h != ci);
    }

    /// Plays a hand card of yours to a zone (face up until End Turn, it can be moved or taken back).
    pub fn stage(&mut self, ci: usize, zone: usize) -> bool {
        if !self.hand[SIDE_P].contains(&ci) || !self.can_play(ci, zone) {
            return false;
        }
        self.remove_from_hand(SIDE_P, ci);
        let c = &mut self.c[ci];
        c.zone = zone as i32;
        c.staged = true;
        c.order = self.play_counter;
        self.play_counter += 1;
        self.energy_left -= cards::card(c.id).cost;
        true
    }

    /// Takes a card played this turn back into its old place in the hand.
    pub fn unstage(&mut self, ci: usize) {
        if self.phase != Phase::Plan || !self.c[ci].staged {
            return;
        }
        self.energy_left += cards::card(self.c[ci].id).cost;
        self.c[ci].zone = -1;
        self.c[ci].staged = false;
        let hs = self.c[ci].hseq;
        let at = self.hand[SIDE_P].iter().position(|&h| self.c[h].hseq > hs).unwrap_or(self.hand[SIDE_P].len());
        self.hand[SIDE_P].insert(at, ci);
    }

    pub fn snap(&mut self) {
        if self.p_snapped || self.phase == Phase::Over {
            return;
        }
        self.p_snapped = true;
        self.stakes *= 2;
        self.say("You SNAPPED! Stakes doubled.");
    }

    pub fn retreat(&mut self) {
        if self.phase == Phase::Over {
            return;
        }
        self.phase = Phase::Over;
        self.result = Outcome::Retreat;
        self.say("You retreated.");
    }

    fn ai_snap(&mut self) {
        if self.ai_snapped || self.turn < 3 || self.turn > 5 {
            return;
        }
        let lead: i32 = (0..3).map(|z| self.zone_power(z, SIDE_AI) - self.zone_power(z, SIDE_P)).sum();
        let strong = self.hand[SIDE_AI].iter().filter(|&&h| cards::card(self.c[h].id).power >= 5).count();
        let score = lead as f32 / 4.0 + strong as f32 * 0.5 + self.winning_zones(SIDE_AI) as f32 - 1.0;
        if score > 1.8 && self.rand() < 0.6 {
            self.ai_snapped = true;
            self.stakes *= 2;
            let msg = format!("{} SNAPPED! Stakes doubled.", self.opponent);
            self.say(msg);
        }
    }

    fn next_turn(&mut self) {
        self.turn += 1;
        if self.turn <= 3 {
            let z = (self.turn - 1) as usize;
            self.zone_rev[z] = true;
            let msg = format!("Zone revealed: {}", cards::ZONES[self.zone[z]].name);
            self.say(msg);
        }
        self.energy = self.turn;
        self.energy_left = self.turn;
        self.draw(SIDE_P, false);
        self.draw(SIDE_AI, false);
        if self.turn == self.max_turns {
            self.stakes *= 2;
            self.say("Final turn! Stakes doubled.");
        }
        self.phase = Phase::Plan;
        let (pw, aw) = (self.winning_zones(SIDE_P), self.winning_zones(SIDE_AI));
        self.first = if pw > aw {
            SIDE_P
        } else if aw > pw {
            SIDE_AI
        } else if self.rand() < 0.5 {
            SIDE_P
        } else {
            SIDE_AI
        };
        self.ai_snap();
    }

    fn shuffle(&mut self, a: &mut [usize]) {
        for i in (1..a.len()).rev() {
            let j = (self.rand_u() % (i as u32 + 1)) as usize;
            a.swap(i, j);
        }
    }

    /// A new battle (Kaigen `b_init`): both decks shuffled, three locations, Game Start abilities, 3 cards each,
    /// turn 1. `player_deck` may be shorter than `DECK_SIZE` (the deck screen allows it).
    pub fn new(kind: BattleKind, opponent: &str, player_deck: &[usize], ai_deck: &[usize], seed: u32) -> Self {
        let mut b = Battle {
            kind,
            opponent: opponent.to_string(),
            c: vec![],
            zone: [0; 3],
            zone_rev: [false; 3],
            hand: [vec![], vec![]],
            deck: [vec![], vec![]],
            turn: 0,
            max_turns: 6,
            energy: 0,
            energy_left: 0,
            phase: Phase::Plan,
            result: Outcome::None,
            stakes: 1,
            p_snapped: false,
            ai_snapped: false,
            first: SIDE_P,
            first_t: 99.0,
            play_counter: 0,
            hseq: 1,
            seed: seed | 1,
            steps: vec![],
            step_i: 0,
            reveal_repeat: 0,
            stage: 0,
            et_side: 0,
            et_zone: 0,
            et_idx: 0,
            wait: 0.0,
            log: vec![],
            sfx_hits: 0,
            sfx_buffs: 0,
            sfx_kills: 0,
            sfx_reveals: 0,
        };
        for s in 0..2 {
            let ids: Vec<usize> = if s == SIDE_P { player_deck.to_vec() } else { ai_deck.to_vec() };
            for id in ids.into_iter().take(cards::DECK_SIZE) {
                let ci = b.new_card(id, s);
                b.deck[s].push(ci);
            }
            let mut d = std::mem::take(&mut b.deck[s]);
            b.shuffle(&mut d);
            b.deck[s] = d;
        }
        let mut zones: Vec<usize> = (0..cards::ZONE_COUNT).collect();
        b.shuffle(&mut zones);
        b.zone = [zones[0], zones[1], zones[2]];
        // Game Start abilities (the cards are still in the decks)
        for s in 0..2 {
            for i in 0..b.deck[s].len() {
                let ci = b.deck[s][i];
                let id = b.c[ci].id;
                if id == cards::QUICKHOP || id == cards::GOLDBRICK {
                    b.to_opening_hand(ci);
                }
                if id == cards::BUILDBEAVER || id == cards::GOLDBRICK {
                    b.c[ci].modp += 2;
                }
                if cards::card(id).kind == CardKind::Start && s == SIDE_P {
                    b.say(format!("Game Start: {}", cards::card(id).name));
                }
            }
        }
        for _ in 0..3 {
            b.draw(SIDE_P, true);
            b.draw(SIDE_AI, true);
        }
        b.next_turn();
        b
    }

    /// The AI's plan (Kaigen `b_ai_plan`): the subset of the hand that spends the most energy (then power), each card
    /// to the zone that needs it. `side` SIDE_AI stages them for the reveal; SIDE_P only returns the plan (the
    /// `battle_auto` test helper plays it through `stage`, the player's path).
    pub fn plan(&mut self, side: usize) -> Vec<(usize, usize)> {
        let hand = self.hand[side].clone();
        let n = hand.len().min(7);
        let budget = if side == SIDE_P { self.energy_left } else { self.energy };
        let mut best_mask = 0usize;
        let mut best_score = -1.0f32;
        for mask in 1..(1usize << n) {
            let (mut cost, mut pow) = (0, 0);
            for (i, &h) in hand.iter().enumerate().take(n) {
                if mask & (1 << i) != 0 {
                    let d = cards::card(self.c[h].id);
                    cost += d.cost;
                    pow += d.power;
                }
            }
            if cost > budget {
                continue;
            }
            let score = (cost * 10 + pow) as f32 + self.rand();
            if score > best_score {
                best_score = score;
                best_mask = mask;
            }
        }
        let mut set: Vec<usize> = (0..n).filter(|i| best_mask & (1 << i) != 0).map(|i| hand[i]).collect();
        // most expensive first (stable like the Kaigen selection sort for distinct costs)
        for i in 0..set.len() {
            for k in i + 1..set.len() {
                if cards::card(self.c[set[k]].id).cost > cards::card(self.c[set[i]].id).cost {
                    set.swap(i, k);
                }
            }
        }
        let foe = other(side);
        let mut pending = [0i32; 3];
        let mut counts = [self.count_at(0, side), self.count_at(1, side), self.count_at(2, side)];
        let mut placed = vec![];
        for ci in set {
            let d = cards::card(self.c[ci].id);
            let mut best_z: Option<usize> = None;
            let mut best_s = -1e9f32;
            for z in 0..3 {
                if counts[z] >= self.capacity(z) {
                    continue;
                }
                if self.zone_rev[z] && cards::ZONES[self.zone[z]].id == "tower" && self.turn >= self.max_turns {
                    continue;
                }
                let diff = self.zone_power(z, side) + pending[z] - self.zone_power(z, foe);
                let mut s = -((diff + d.power - 3) as f32).abs();
                if diff > 10 {
                    s -= 6.0;
                }
                if diff < -14 {
                    s -= 6.0;
                }
                if self.zone_rev[z] {
                    let (oz, orv) = (self.c[ci].zone, self.c[ci].revealed);
                    self.c[ci].zone = z as i32;
                    self.c[ci].revealed = true;
                    s += self.zone_bonus(z, ci) as f32 * 2.0;
                    self.c[ci].zone = oz;
                    self.c[ci].revealed = orv;
                }
                let id = self.c[ci].id;
                if id == cards::ZEPHYRROW {
                    s += if counts[z] == 0 { 6.0 } else { -6.0 };
                }
                if d.kind == CardKind::Reveal && self.zone_rev[z] && cards::ZONES[self.zone[z]].id == "library" {
                    s -= 4.0;
                }
                if matches!(id, cards::VOLTAIL | cards::THUNDRAKE | cards::PHANTOMULE | cards::CINDERPUP | cards::GLACIOMOTH) && !self.cards_at(z as i32, foe).is_empty() {
                    s += 4.0;
                }
                s += self.rand() * 2.0;
                if s > best_s {
                    best_s = s;
                    best_z = Some(z);
                }
            }
            let Some(z) = best_z else { continue };
            counts[z] += 1;
            pending[z] += d.power;
            placed.push((ci, z));
            if side == SIDE_AI {
                self.remove_from_hand(side, ci);
                let c = &mut self.c[ci];
                c.zone = z as i32;
                c.staged = true;
                c.order = self.play_counter;
                self.play_counter += 1;
            }
        }
        placed
    }

    /// End Turn: the AI chooses its Activates and plays, then everything is revealed step by step (`update`).
    pub fn end_turn(&mut self) {
        if self.phase != Phase::Plan {
            return;
        }
        self.phase = Phase::Reveal;
        for i in 0..self.c.len() {
            if self.c[i].owner == SIDE_AI && self.can_activate(i) {
                self.activate(i);
            }
        }
        self.plan(SIDE_AI);
        self.first_t = 0.0;
        let msg = if self.first == SIDE_P { "You reveal first.".to_string() } else { format!("{} reveals first.", self.opponent) };
        self.say(msg);
        // each side: its plays and Activates in the order they were done; the first side first
        self.steps.clear();
        for pass in 0..2 {
            let side = if pass == 0 { self.first } else { other(self.first) };
            let start = self.steps.len();
            for i in 0..self.c.len() {
                let c = &self.c[i];
                if c.owner != side || !c.alive {
                    continue;
                }
                if c.staged && c.zone >= 0 {
                    self.steps.push(Step { kind: StepKind::Reveal, card: i, at: c.order });
                }
                if c.act_order >= 0 {
                    self.steps.push(Step { kind: StepKind::Act, card: i, at: c.act_order });
                }
            }
            self.steps[start..].sort_by_key(|s| s.at);
        }
        self.step_i = 0;
        self.reveal_repeat = 0;
        self.stage = 0;
        self.wait = 0.35;
    }

    fn finish(&mut self) {
        self.phase = Phase::Over;
        let (pw, aw) = (self.winning_zones(SIDE_P), self.winning_zones(SIDE_AI));
        self.result = if pw != aw {
            if pw > aw {
                Outcome::Win
            } else {
                Outcome::Lose
            }
        } else {
            let tp: i32 = (0..3).map(|z| self.zone_power(z, SIDE_P)).sum();
            let ta: i32 = (0..3).map(|z| self.zone_power(z, SIDE_AI)).sum();
            if tp > ta {
                Outcome::Win
            } else if ta > tp {
                Outcome::Lose
            } else {
                Outcome::Draw
            }
        };
        self.say(match self.result {
            Outcome::Win => "Victory!",
            Outcome::Lose => "Defeat...",
            _ => "Draw.",
        });
    }

    /// Every frame: the timers, and the reveal's next step once its pause is over.
    pub fn update(&mut self, dt: f32) {
        self.first_t += dt;
        for c in &mut self.c {
            c.fx_t += dt;
            c.pulse_t += dt;
            c.die_t += dt;
            c.draw_t += dt;
        }
        if self.phase != Phase::Reveal {
            return;
        }
        self.wait -= dt;
        if self.wait > 0.0 {
            return;
        }
        if self.stage == 0 {
            if self.step_i >= self.steps.len() {
                self.stage = 1;
                self.et_side = 0;
                self.et_zone = 0;
                self.et_idx = 0;
                for c in &mut self.c {
                    c.staged = false;
                }
                return;
            }
            let st = self.steps[self.step_i];
            let ci = st.card;
            if st.kind == StepKind::Act {
                self.c[ci].act_order = -1;
                self.step_i += 1;
                let c = &self.c[ci];
                if !c.activated && c.alive && c.zone >= 0 {
                    self.c[ci].activated = true;
                    self.c[ci].pulse_t = 0.0;
                    let who = if self.c[ci].owner == SIDE_P { "You".to_string() } else { self.opponent.clone() };
                    let msg = format!("{who} activated {}.", self.name(ci));
                    self.say(msg);
                    self.activate_fx(ci);
                    self.wait = 0.65;
                }
                return;
            }
            if self.reveal_repeat == 0 {
                if !self.c[ci].alive || self.c[ci].zone < 0 {
                    self.step_i += 1; // destroyed meanwhile
                    return;
                }
                let c = &mut self.c[ci];
                c.staged = false;
                c.revealed = true;
                c.pulse_t = 0.0;
                self.sfx_reveals += 1;
            }
            let times = if cards::card(self.c[ci].id).kind == CardKind::Reveal { self.reveal_times(ci) } else { 0 };
            if self.reveal_repeat < times {
                if self.reveal_repeat > 0 {
                    // doubled (Spotlight Stage, Echowl): again, after the first one has played out
                    self.c[ci].pulse_t = 0.27;
                    let msg = format!("{} triggers again!", self.name(ci));
                    self.say(msg);
                }
                self.on_reveal(ci);
                self.reveal_repeat += 1;
                // the next one waits for this one's ball and impact; after the last, as usual
                self.wait = if self.reveal_repeat < times { 1.1 } else { 0.65 };
                if self.reveal_repeat >= times {
                    self.reveal_repeat = 0;
                    self.step_i += 1;
                }
                return;
            }
            self.reveal_repeat = 0;
            self.step_i += 1;
            self.wait = 0.65;
            return;
        }
        if self.stage == 1 {
            // End of Turn: the first side, then the other, zone by zone
            while self.et_side < 2 {
                let side = if self.et_side == 0 { self.first } else { other(self.first) };
                let arr = self.cards_at(self.et_zone as i32, side);
                while self.et_idx < arr.len() {
                    let ci = arr[self.et_idx];
                    self.et_idx += 1;
                    if !Self::has_end_turn(self.c[ci].id) {
                        continue;
                    }
                    let times = if self.zone_rev[self.et_zone] && cards::ZONES[self.zone[self.et_zone]].id == "clock" { 2 } else { 1 };
                    for _ in 0..times {
                        self.end_turn_fx(ci);
                    }
                    self.wait = 0.4;
                    return;
                }
                self.et_idx = 0;
                self.et_zone += 1;
                if self.et_zone >= 3 {
                    self.et_zone = 0;
                    self.et_side += 1;
                }
            }
            self.stage = 2;
        }
        if self.turn >= self.max_turns {
            self.finish();
        } else {
            self.next_turn();
        }
    }

    /// The cards of a side at a zone in the order they were put there (a new card never pushes an older one out of
    /// its spot), staged ones too.
    pub fn zone_order(&self, z: usize, side: usize) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.c.len()).filter(|&i| {
            let c = &self.c[i];
            c.alive && c.zone == z as i32 && c.owner == side
        })
        .collect();
        v.sort_by_key(|&i| self.c[i].order);
        v
    }

    pub fn any_staged(&self) -> bool {
        self.c.iter().any(|c| c.staged)
    }

    /// Nothing left to do this turn: no card can go anywhere and no Activate is ready.
    pub fn no_moves(&self) -> bool {
        if self.phase != Phase::Plan {
            return false;
        }
        for &h in &self.hand[SIDE_P] {
            for z in 0..3 {
                if self.can_play(h, z) {
                    return false;
                }
            }
        }
        !(0..self.c.len()).any(|i| self.c[i].owner == SIDE_P && self.can_activate(i))
    }
}

// ---------------------------------------------------------------- what the opponent says

pub struct Taunt {
    pub text: &'static str,
    pub voice: Option<&'static str>,
}

const fn t(text: &'static str, voice: &'static str) -> Taunt {
    Taunt { text, voice: Some(voice) }
}
const fn tn(text: &'static str) -> Taunt {
    Taunt { text, voice: None }
}

/// BattleView.svelte LINES (Kaigen `cg_line`); `what`: 0 start, 1 turn, 2 snap, 3 the opponent won, 4 the opponent
/// lost. Dex and Lumi are voiced.
pub fn line(k: BattleKind, what: usize, r: u32) -> &'static Taunt {
    static RIVAL: [&[Taunt]; 5] = [
        &[t("Grandma gave YOU a deck? This will be quick.", "e0b185df"), t("Ready to lose?", "2ea3ea6a")],
        &[t("Is that all you got?", "38c41100"), t("My Beasts are way cooler.", "241a4bbf"), t("Watch this!", "dd0fe3a0"), t("Hmm\u{2026} lucky draw.", "f2108d20")],
        &[t("SNAP! Double or nothing, slowpoke!", "a5a25e9f"), t("SNAP! I like my odds.", "335ad1f3")],
        &[t("Told you my deck was better!", "c31240c9")],
        &[t("No way\u{2026} my deck was perfect!", "e6a74568")],
    ];
    static PRACTICE: [&[Taunt]; 5] = [
        &[t("Let\u{2019}s have a fun practice round!", "7e9bc202")],
        &[t("Nice move!", "49205884"), t("Ooh, careful with that zone.", "09c03c86"), t("Remember: win 2 of 3!", "bdf62197")],
        &[t("Snap! Let\u{2019}s make it exciting!", "760dc0c6")],
        &[t("Good try! You\u{2019}re getting better.", "1f93971b")],
        &[t("Wow, you learn fast!", "45311098")],
    ];
    static PIP: [&[Taunt]; 5] = [
        &[tn("Ready, set, GO! My deck is the best!"), tn("I practised with my big brother's cards!")],
        &[tn("Hehe, too slow!"), tn("Watch my fishy ones!"), tn("Is it my turn yet?"), tn("Ooh, sneaky!")],
        &[tn("SNAP! I'm super fast AND super brave!")],
        &[tn("I win, I win! Hehe!")],
        &[tn("No fair! Again! Again!")],
    ];
    static WILD: [&[Taunt]; 5] = [
        &[tn("*rustle rustle*"), tn("Grrrr!")],
        &[tn("*snarl*"), tn("*brick noises*"), tn("Rrraah!")],
        &[tn("*SNAP!* (it bares its teeth)")],
        &[tn("*victory howl*")],
        &[tn("*runs back into the grass*")],
    ];
    let set = match k {
        BattleKind::Rival => &RIVAL,
        BattleKind::Practice => &PRACTICE,
        BattleKind::Pip => &PIP,
        BattleKind::Wild => &WILD,
    };
    let l = set[what.min(4)];
    &l[r as usize % l.len()]
}
