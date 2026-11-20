//! Dialogues (Kaigen `st_dialog.c`): a speaker box at the bottom, lines advanced with E / Space / Enter,
//! a click on the box or the touch Use button, up to three choices at the end (←/→ or A/D pick, E
//! confirms, a click / tap picks). Every line the three.js version had voiced plays its recording.
//!
//! Hooks for later parts:
//! * `dialog_for(DialogId, &GameSave)` builds the lines as they depend on the progress (all the Kaigen
//!   dialogues are here: Mom, the Professor in the lab, Lumi, the Deck-o-Matic, Stonecrest, the battle
//!   outcomes, ...). Open one with `DialogState::open`.
//! * When a dialog closes, `DialogState::update` returns the chosen [`DialogAction`] (or the dialog's
//!   `on_end`); the play screen acts on it (`PlayScreen::dialog_action`). Battles, hide and seek and the
//!   quest steps are such actions: they arrive there and are logged until their part implements them.

#![allow(dead_code)] // hooks for later parts (see the module doc)

use crate::save::GameSave;
use crate::sound::Sound;
use engine::prelude::*;

/// Brick Beast cards in the game (the Professor counts them).
pub const CARD_POOL: u32 = 34;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogAction {
    None,
    HideSeek,
    Tag,
    Sleep,
    GiveStarter,
    BattleRival,
    BattlePractice,
    DexScared,
    OpenGate,
    QuestPart,
    BattlePip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogId {
    IntroKeys,
    IntroTouch,
    Mom,
    Prof,
    Dex,
    Lumi,
    Pip,
    Gate,
    ProfAfter,
    DexScared,
    DexWon,
    DexLost,
    LumiWon,
    LumiLost,
    Machine,
    Knight,
    Baker,
    PipNotYet,
    PipWon,
    PipLost,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub text: String,
    pub voice: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct Dialog {
    pub id: DialogId,
    pub speaker: &'static str,
    pub lines: Vec<Line>,
    pub choices: Vec<(&'static str, DialogAction)>,
    pub on_end: DialogAction,
}

fn l(text: &str, voice: &'static str) -> Line {
    Line { text: text.into(), voice: if voice.is_empty() { None } else { Some(voice) } }
}

fn d(id: DialogId, speaker: &'static str, lines: Vec<Line>) -> Dialog {
    Dialog { id, speaker, lines, choices: vec![], on_end: DialogAction::None }
}

const QUILL: &str = "Prof. Hazel Quill";

/// The lines as the three.js version had them before the first cards (Kaigen `dialog_script`).
pub fn dialog_script(id: DialogId) -> Dialog {
    use DialogId::*;
    match id {
        IntroKeys => d(id, "You", vec![
            l("*yawn* What a beautiful morning in Brickhollow.", "91b19939"),
            l("I should go downstairs and talk to Mom in the kitchen. WASD / arrow keys to walk, E or Space to talk.", "4f858f02"),
        ]),
        IntroTouch => d(id, "You", vec![
            l("*yawn* What a beautiful morning in Brickhollow.", "91b19939"),
            l("I should go downstairs and talk to Mom in the kitchen. Drag the stick to walk and tap the talk button.", "459836e6"),
        ]),
        Mom => d(id, "Mom", vec![
            l("Morning, sweetie! Professor Quill was looking for you earlier.", "10a293f7"),
            l("Her lab is the big tan building with the solar panels \u{2014} just across the road to the south-east.", "85d4a2ce"),
        ]),
        Prof => d(id, QUILL, vec![
            l("Ah, there you are! Welcome to Quill Lab.", "df9ece56"),
            l("All around Brickhollow live BRICK BEASTS \u{2014} little creatures snapped together from living bricks!", "97e8d994"),
            l("Trainers here battle with decks of 12 Beast cards, fighting over 3 zones for 6 turns.", "faa6ffcd"),
            l("Did you know? Glimmerlings fold light into tiny bricks. Fascinating creatures.", "c83f36f4"),
        ]),
        Dex => d(id, "Dex", vec![
            l("Hey, slowpoke! You don't even have a deck yet.", "14add442"),
            l("Go see Grandma Quill at the lab. Then I'll crush you!", "88739c53"),
        ]),
        Lumi => Dialog {
            choices: vec![("Got it!", DialogAction::None)],
            ..d(id, "Lumi", vec![
                l("Hi! I'm Lumi, Dex's sister. Want a quick lesson?", "dbc3cccf"),
                l("Each turn you get energy equal to the turn number \u{2014} 1 on turn 1, up to 6 on turn 6.", "e99643e2"),
                l("Play cards into the 3 zones. Zones reveal one by one on turns 1 to 3 and each has its own effect.", "1e658f85"),
                l("Both players plan in secret, then cards reveal. Whoever is ahead reveals first.", "e7930b4e"),
                l("Cards come in ability types: On Reveal fires when the card flips, Ongoing works all the time.", "7abe2838"),
                l("Activate cards can be used once \u{2014} tap them on the board. End of Turn cards trigger after every turn, and Game Start cards do their thing before the first draw.", "064e0ba2"),
                l("Tap any card to see it big, and tap a location to read what it does!", "fb9d3419"),
                l("After turn 6, win 2 of the 3 zones to win! Hit SNAP to double the stakes if you feel confident.", "251412fd"),
            ])
        },
        Knight => d(id, "Sir Bricksworth", vec![
            l("Welcome to Stonecrest, traveller! I guard the old castle.", ""),
            l("It was built from the very first bricks of the valley. Some say Brick Beasts still live in its towers.", ""),
            l("The gate is closed for now. Come back when you are a famous trainer!", ""),
        ]),
        Baker => d(id, "Marla", vec![
            l("Oh, a visitor from Brickhollow! You came all the way through the forest?", ""),
            l("Have a look around. Our fountain is the prettiest in the whole valley.", ""),
        ]),
        Pip => Dialog {
            choices: vec![("Play hide and seek", DialogAction::HideSeek), ("Play tag", DialogAction::Tag)],
            ..d(id, "Pip", vec![
                l("Psst! I saw something wiggle in the tall grass up north!", "afa6d9b1"),
                l("Also\u{2026} there are 15 golden studs hidden around Brickhollow. Some are even inside houses! Find them all and something legendary happens.", "f19b68ac"),
                l("My big brother says there are 34 different Brick Beasts. I only know the fishy ones.", "153eb44f"),
                l("Hey, I'm bored! Wanna play hide and seek? Or tag? I'm super fast!", ""),
            ])
        },
        _ => d(Gate, QUILL, vec![
            l("Wait! Don't go out there!", "c77df687"),
            l("Wild Brick Beasts live in the tall grass. You can't go without any Beast cards!", "7cf5da3f"),
            l("Come to my lab first. I'll give you your very first cards.", "22139e70"),
        ]),
    }
}

/// "You have N of 34 Brick Beast cards." as recorded for every N.
fn prof_count_voice(n: u32) -> &'static str {
    const IDS: [&str; 35] = [
        "8b373374", "925f7191", "cacfe5d2", "adec45f7", "cfe9e390", "3266cefd", "fbb93e1e", "7e9955e3", "2df970fc", "6ed80a59", "dc71ff0d", "9c57c1a0",
        "ce9262b3", "2c64cb2e", "e2242121", "aff263c4", "a3629d07", "4c501aa2", "3bc9d495", "6088ca08", "c512a37c", "05f13cd9", "85280dda", "6abac11f",
        "ba89a838", "8c9677c5", "d7a87066", "85ce9c2b", "225065f4", "2978a411", "0f30fe4f", "09b26b0a", "6afa0609", "b6afd02c", "522b0d4b",
    ];
    IDS[(n as usize).min(34)]
}

/// The lines as they depend on your progress (Kaigen `dialog_for`, the three.js talk()).
pub fn dialog_for(id: DialogId, s: &GameSave) -> Dialog {
    use DialogAction as A;
    use DialogId::*;
    let mut dl = dialog_script(id);
    let has_voxelmon = s.voxelmon > 0;
    match id {
        Mom if s.has_deck => {
            dl.lines = vec![
                if s.wins > 0 { l("So many wins already? I'm so proud of you!", "2dcf3ee1") } else { l("Off on an adventure? Be careful in that tall grass!", "e212973d") },
                l("Remember: you can rebuild your deck any time with the Deck button (C).", "33a0ab32"),
                l("And snap only when you feel lucky. Your father lost a whole bucket of studs that way!", "79f91424"),
            ];
        }
        Prof => {
            if !s.has_deck && !s.has_part {
                if !s.quest_part {
                    dl.lines = vec![
                        l("Ah, there you are! Welcome to Quill Lab.", "df9ece56"),
                        l("All around Brickhollow live BRICK BEASTS \u{2014} little creatures snapped together from living bricks!", "97e8d994"),
                        l("Trainers here battle with decks of 12 Beast cards, fighting over 3 zones for 6 turns.", "faa6ffcd"),
                        l("I wanted to print your very first deck with my Deck-o-Matic over there... but it only blinks MISSING PART!", ""),
                        l("Its Stud Core is gone. I dropped it while diving in the pond. Yes, the pond: it is the way into a deep, flooded cave.", ""),
                        l("My old diving suit must be somewhere in this lab. Find it, dive down and bring me the Stud Core!", ""),
                    ];
                    dl.on_end = A::QuestPart;
                } else if !s.has_suit {
                    dl.lines = vec![l("Found my diving suit yet? It is in here somewhere... in a corner nobody ever looks into, I suppose.", "")];
                } else {
                    dl.lines = vec![
                        l("The suit fits you! Go to the pond and dive in.", ""),
                        l("Deep down the Stud Core glows yellow. Follow the light, and mind the dark parts.", ""),
                    ];
                }
            } else if !s.has_deck {
                dl.lines = vec![
                    l("The Stud Core! You really found it down there!", ""),
                    l("Let me put it in... *clank* *whirr* ... the Deck-o-Matic is printing!", ""),
                    l("Here are 18 cards to start your collection. I've already built you a deck of 12!", "83c29583"),
                    l("Wild Beasts hide in the tall grass on Route 1, north of the village. Beat them to collect new cards!", "a633cd6e"),
                ];
                dl.on_end = A::GiveStarter;
            } else {
                let n = (s.collection & ((1u64 << CARD_POOL) - 1)).count_ones();
                let tail = if n >= CARD_POOL { "A complete collection \u{2014} incredible!" } else { "Keep exploring Route 1!" };
                dl.lines = vec![
                    Line { text: format!("You have {n} of {CARD_POOL} Brick Beast cards. {tail}"), voice: Some(prof_count_voice(n)) },
                    l("Did you know? Glimmerlings fold light into tiny bricks. Fascinating creatures.", "c83f36f4"),
                ];
            }
        }
        Machine => {
            dl = d(id, "Deck-o-Matic", if !s.has_deck && !s.has_part {
                vec![l("*beep* ERROR: MISSING PART. Stud Core not found.", ""), l("Insert a Stud Core to print Beast decks. *beep*", "")]
            } else if !s.has_deck {
                vec![l("*beep* Stud Core detected. Waiting for Prof. Quill.", "")]
            } else {
                vec![l("*whirr* READY. Decks printed today: 1. Have fun!", "")]
            });
        }
        ProfAfter => {
            dl = d(id, QUILL, vec![
                l("Lumi, Dex's sister, knows the rules by heart \u{2014} visit their big house in the east for a practice match.", "2a919bfb"),
                l("And watch out for my grandson Dex. He has been itching for a battle all week!", "5facc525"),
            ]);
        }
        Dex if s.has_deck => {
            dl.lines = if s.rival_wins > 0 {
                vec![l("Back for more? This time I upgraded my deck!", "755b07b2"), l("Ready?", "bd46fed4")]
            } else {
                vec![l("So Grandma gave you a deck, huh? Mine is WAY cooler.", "676ff784"), l("Let's battle \u{2014} right here, right now!", "4493640a")]
            };
            dl.choices = vec![("Battle!", A::BattleRival), ("Not now", A::DexScared)];
        }
        DexScared => dl = d(id, "Dex", vec![l("Scared? Ha! Come back when you are ready.", "e4e22ae0")]),
        DexWon => dl = d(id, "Dex", vec![l("WHAT?! No way... my deck was perfect!", "1d5e5e86"), l("Fine. You win this time. I'm going to train on Route 1!", "e454c49a")]),
        DexLost => dl = d(id, "Dex", vec![l("Ha! Told you my deck was cooler.", "8c7da71a"), l("Come back when you have some better cards!", "d12633cc")]),
        Pip if s.has_deck => {
            dl.lines[3] = l("Hey, I'm bored! Hide and seek? Tag? Or... a card battle? My deck is secretly the best!", "");
            dl.choices = vec![("Hide and seek", A::HideSeek), ("Tag", A::Tag), ("Card battle!", A::BattlePip)];
        }
        PipNotYet => {
            dl = d(id, "Pip", vec![
                l("A card battle? Nooo, that's boring. Hide and seek first!", ""),
                l("Play one round with me, then I'll show you my deck. It's even better than my brother's!", ""),
            ]);
            dl.choices = vec![("Hide and seek", A::HideSeek), ("Later", A::None)];
        }
        PipWon => dl = d(id, "Pip", vec![l("Whaaat? You beat my super deck?!", ""), l("Okay, okay... you're good. But I'm still faster than you!", "")]),
        PipLost => dl = d(id, "Pip", vec![l("Hehe! Told you, my deck is the best!", ""), l("Come back when you've got stronger cards. I'll be by the pond!", "")]),
        LumiWon => dl = d(id, "Lumi", vec![l("Wow, you learn fast! Here, keep that card.", "f8de2d54")]),
        LumiLost => dl = d(id, "Lumi", vec![l("Good try! Watch which zones you can still win and focus there.", "2a3279f7")]),
        Lumi if s.has_deck => dl.choices = vec![("Practice battle", A::BattlePractice), ("Thanks!", A::None)],
        Gate if s.has_deck && !has_voxelmon => {
            dl.lines = vec![
                l("Hold on! Cards alone won't protect you out there.", "eeac3b70"),
                l("The wild Beasts on Route 1 are real Voxelmon, made of living bricks!", "5376fc4a"),
                l("Win card battles to earn EXP, and upgrade your cards in the Deck menu.", "d85cccae"),
                l("Once a six energy card reaches its final stage, it becomes a real Voxelmon. Then you may pass!", "13e542b8"),
            ];
        }
        Gate if s.has_deck => {
            dl.lines = vec![l("Oh my! A real Voxelmon! Then you are ready.", "e99f9d41"), l("Press 1 and 2 in battle to use its attacks. Good luck out there!", "b9510f9e")];
            dl.on_end = A::OpenGate;
        }
        _ => {}
    }
    dl
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[derive(Default)]
pub struct DialogState {
    pub d: Option<Dialog>,
    pub index: usize,
    /// Keyboard selection of a choice.
    pub hover: usize,
    /// Lines shown since the start (for tests).
    pub lines_shown: u32,
    /// The last dialog that closed and what it chose (for tests).
    pub last_closed: Option<(DialogId, DialogAction)>,
}

impl DialogState {
    pub fn is_open(&self) -> bool {
        self.d.is_some()
    }

    fn voice_line(&mut self, ctx: &mut Ctx, sound: &mut Sound) {
        let v = self.d.as_ref().and_then(|d| d.lines.get(self.index)).and_then(|l| l.voice);
        sound.voice(ctx, v);
        self.lines_shown += 1;
    }

    pub fn open(&mut self, ctx: &mut Ctx, sound: &mut Sound, d: Dialog) {
        log_info!("dialog: {:?} ({} lines)", d.id, d.lines.len());
        // the later lines load while the first one plays
        for l in d.lines.iter().skip(1) {
            if let Some(v) = l.voice {
                sound.prefetch_voice(ctx, v);
            }
        }
        self.d = Some(d);
        self.index = 0;
        self.hover = 0;
        self.voice_line(ctx, sound);
    }

    fn close(&mut self, ctx: &mut Ctx, sound: &mut Sound, chosen: Option<DialogAction>) -> DialogAction {
        sound.stop_voice(ctx);
        let Some(d) = self.d.take() else { return DialogAction::None };
        let act = chosen.unwrap_or(d.on_end);
        self.last_closed = Some((d.id, act));
        act
    }

    fn at_choices(&self) -> bool {
        self.d.as_ref().is_some_and(|d| self.index + 1 >= d.lines.len() && !d.choices.is_empty())
    }

    /// Every frame while open: `advance` = E, Space, Enter or the Use button; `actions` = the UI clicks
    /// (`dlg_next`, `dlg_choice:N`). Returns the action when the dialog closed this frame.
    pub fn update(&mut self, ctx: &mut Ctx, sound: &mut Sound, advance: bool, actions: &[String]) -> Option<DialogAction> {
        let d = self.d.as_ref()?;
        let n = d.choices.len();
        let last = self.index + 1 >= d.lines.len();
        let clicked = actions.iter().any(|a| a == "dlg_next");
        let picked = actions.iter().find_map(|a| a.strip_prefix("dlg_choice:").and_then(|n| n.parse::<usize>().ok()));
        let esc_key = ctx.input.key_pressed("Escape");
        if last && n > 0 {
            let inp = &ctx.input;
            if inp.key_pressed("ArrowLeft") || inp.key_pressed("KeyA") || inp.key_pressed("ArrowUp") || inp.key_pressed("KeyW") {
                self.hover = self.hover.saturating_sub(1);
            }
            if inp.key_pressed("ArrowRight") || inp.key_pressed("KeyD") || inp.key_pressed("ArrowDown") || inp.key_pressed("KeyS") {
                self.hover = (self.hover + 1).min(n - 1);
            }
            let pick = picked.filter(|p| *p < n).or(if advance { Some(self.hover) } else { None });
            if let Some(p) = pick {
                let act = d.choices[p].1;
                return Some(self.close(ctx, sound, Some(act)));
            }
            if esc_key {
                return Some(self.close(ctx, sound, Some(DialogAction::None)));
            }
            return None;
        }
        if esc_key {
            return Some(self.close(ctx, sound, None));
        }
        if advance || clicked {
            if last {
                return Some(self.close(ctx, sound, None));
            }
            self.index += 1;
            self.voice_line(ctx, sound);
        }
        None
    }

    /// The box (slot "dialog"); `night` 0..1 darkens it like the Kaigen `ui_night_mix`.
    pub fn draw(&self, ctx: &mut Ctx, touch: bool, night: f32) {
        let Some(d) = &self.d else {
            ctx.ui.set("dialog", "");
            return;
        };
        let line = d.lines.get(self.index).map(|l| esc(&l.text)).unwrap_or_default();
        let foot = if self.at_choices() {
            let b: String = d
                .choices
                .iter()
                .enumerate()
                .map(|(i, (t, _))| format!("<button class='{}' data-action='dlg_choice:{i}'>{}</button>", if i == self.hover { "sel" } else { "" }, esc(t)))
                .collect();
            format!("<div class='bh-choices'>{b}</div>")
        } else {
            format!("<div class='bh-cont'>{}</div>", if touch { "tap to continue" } else { "E / Space / click to continue" })
        };
        let action = if self.at_choices() { "" } else { " data-action='dlg_next'" };
        ctx.ui.set(
            "dialog",
            &format!(
                "<div class='bh-dialog{}{}'{action}><div class='bh-speaker'>{}</div><div class='bh-line'>{line}</div>{foot}</div>",
                if touch { " touch" } else { "" },
                if night > 0.5 { " night" } else { "" },
                esc(d.speaker)
            ),
        );
    }
}
