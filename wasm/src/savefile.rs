//! The save game as a file: the menu's Export writes the whole [`GameSave`] as JSON and the browser downloads it
//! (`brickhollow-save-<date>.json`); Import opens the browser's file picker, reads the chosen file and, when it is a
//! Brickhollow save, makes it the save and reloads the page (every screen starts again from it).
//!
//! The picker and the reader are asynchronous: the chosen file's text lands in [`SaveFile::pending`] and
//! [`SaveFile::poll`] (every frame, from the menu) applies it. A file that is not a save (no JSON object, or JSON
//! that is not a save's) is refused with a message and changes nothing. Fields the file lacks take their defaults
//! (`#[serde(default)]` on `GameSave`), so older exports still load.

use crate::save::{GameSave, Save};
use engine::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Default)]
pub struct SaveFile {
    /// The text of a picked file, once read (set by the browser's callbacks)
    pending: Rc<RefCell<Option<String>>>,
    /// What the last export / import did (the menu shows it; tests read it)
    pub status: Option<String>,
}

/// A file text that is a Brickhollow save, or why not.
pub fn parse(text: &str) -> Result<GameSave, String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("not JSON ({e})"))?;
    let obj = v.as_object().ok_or("not a save (no JSON object)")?;
    // (a save has these: any JSON object would otherwise load as a new game with all defaults)
    if !obj.contains_key("studs") || !obj.contains_key("plays") {
        return Err("not a Brickhollow save".into());
    }
    serde_json::from_value::<GameSave>(v).map_err(|e| format!("not a Brickhollow save ({e})"))
}

impl SaveFile {
    /// The save as pretty JSON (also the debug command `save_export`).
    pub fn json(save: &Save) -> String {
        serde_json::to_string_pretty(&save.data).unwrap_or_else(|_| "{}".into())
    }

    /// Downloads the save as a .json file.
    pub fn export(&mut self, save: &Save) {
        let json = Self::json(save);
        let name = format!("brickhollow-save-{}.json", today());
        self.status = Some(match download(&name, &json) {
            Ok(()) => format!("Saved as {name}"),
            Err(e) => format!("Export failed: {e}"),
        });
    }

    /// Opens the file picker; the file is applied by `poll` once it has been read.
    pub fn pick(&mut self) {
        let pending = self.pending.clone();
        self.status = match pick_file(pending) {
            Ok(()) => None,
            Err(e) => Some(format!("Import failed: {e}")),
        };
    }

    /// Every frame: a read file becomes the save (written, then the page reloads). Returns true when it did.
    pub fn poll(&mut self, ctx: &mut Ctx, save: &mut Save) -> bool {
        let Some(text) = self.pending.borrow_mut().take() else { return false };
        self.apply(ctx, save, &text, true)
    }

    /// Makes `text` the save (the debug command `save_import` passes `reload: false`).
    pub fn apply(&mut self, ctx: &mut Ctx, save: &mut Save, text: &str, reload: bool) -> bool {
        match parse(text) {
            Ok(g) => {
                save.data = g;
                save.flush(ctx);
                self.status = Some("Save loaded".into());
                log_info!("save: imported a save file (studs {}, golden {})", save.data.studs, save.data.golden_count());
                if reload {
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().reload();
                    }
                }
                true
            }
            Err(e) => {
                self.status = Some(format!("Import failed: {e}"));
                log_warn!("save: import refused: {e}");
                false
            }
        }
    }
}

/// yyyy-mm-dd of the browser's clock.
fn today() -> String {
    let d = js_sys::Date::new_0();
    format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
}

fn download(name: &str, text: &str) -> Result<(), String> {
    let err = |e: JsValue| format!("{e:?}");
    let doc = web_sys::window().and_then(|w| w.document()).ok_or("no document")?;
    let parts = js_sys::Array::of1(&JsValue::from_str(text));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("application/json");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts).map_err(err)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(err)?;
    let a: web_sys::HtmlAnchorElement = doc.create_element("a").map_err(err)?.dyn_into().map_err(|_| "no anchor")?;
    a.set_href(&url);
    a.set_download(name);
    a.click();
    // (the download has started from the object URL: it can go a little later)
    let revoke = Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    });
    if let Some(w) = web_sys::window() {
        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(revoke.unchecked_ref(), 10_000);
    }
    Ok(())
}

/// A hidden file input, clicked (the menu tap is the user gesture the browser wants: the click comes within the
/// same frame's actions); its file's text goes into `pending`.
fn pick_file(pending: Rc<RefCell<Option<String>>>) -> Result<(), String> {
    let err = |e: JsValue| format!("{e:?}");
    let doc = web_sys::window().and_then(|w| w.document()).ok_or("no document")?;
    let input: web_sys::HtmlInputElement = doc.create_element("input").map_err(err)?.dyn_into().map_err(|_| "no input")?;
    input.set_type("file");
    input.set_accept(".json,application/json");
    let picked = input.clone();
    let on_change = Closure::once_into_js(move || {
        let Some(file) = picked.files().and_then(|l| l.get(0)) else { return };
        let Ok(reader) = web_sys::FileReader::new() else { return };
        let r2 = reader.clone();
        let on_load = Closure::once_into_js(move || {
            if let Some(text) = r2.result().ok().and_then(|v| v.as_string()) {
                *pending.borrow_mut() = Some(text);
            }
        });
        reader.set_onload(Some(on_load.unchecked_ref()));
        let _ = reader.read_as_text(&file);
    });
    input.set_onchange(Some(on_change.unchecked_ref()));
    input.click();
    Ok(())
}
