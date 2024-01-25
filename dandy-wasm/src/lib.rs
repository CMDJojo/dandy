//! Wasm bindings for the dandy library. These bindings can be imported and accessed via JavaScript. The DFAs/NFAs are
//! not exposed to JavaScript, rather when loading them using load_dfa() and load_nfa() (which parses its input and
//! generates an DFA/NFA respectively), the DFA or NFA is stored in the wasm runtime, and an unique key is returned
//! which is used to reference the DFA/NFA later on. The DFA/NFA is kept loaded until delete_dfa()/delete_dfa() is
//! called. Example usage:
//! ```js
//! import init, { load_nfa, nfa_to_dfa, delete_nfa, dfa_to_table, delete_dfa } from './web_bindings/dandy_wasm.js';
//! function convert() {
//!     // make sure to call init() first!
//!     let input = document.getElementById("input");
//!     let nfa = load_nfa(input); // can be surrounded by try/catch to catch parsing errors
//!     let dfa = nfa_to_dfa(nfa);
//!     let table = dfa_to_table(dfa);
//!     delete_nfa(nfa);
//!     delete_dfa(dfa);
//!     return table;
//! }
//! ```

use dandy::dfa::parse::DfaParseError;
use dandy::dfa::Dfa;
use dandy::nfa::parse::NfaParseError;
use dandy::nfa::Nfa;
use dandy::regex::Regex;
use dandy_draw::canvas::CanvasDrawer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::RangeFrom;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

thread_local! {
    static DFA_MAP: RefCell<HashMap<usize, Dfa>> = RefCell::default();
    static NFA_MAP: RefCell<HashMap<usize, Nfa>> = RefCell::default();
    static REGEX_MAP: RefCell<HashMap<usize, Regex>> = RefCell::default();
    static KEYGEN: RefCell<RangeFrom<usize>> = RefCell::from(1usize..);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn regex_to_nfa(regex: usize) -> Option<usize> {
    REGEX_MAP
        .with_borrow_mut(|map| map.remove(&regex))
        .map(|regex| push_nfa(regex.to_nfa()))
}

#[wasm_bindgen]
pub fn minimize_dfa(dfa: usize) -> bool {
    DFA_MAP.with_borrow_mut(|map| map.get_mut(&dfa).map(|dfa| dfa.minimize()).is_some())
}

#[wasm_bindgen]
pub fn draw_dfa(dfa: usize, canvas_id: &str) -> bool {
    let Some(dfa) = DFA_MAP.with_borrow(|map| map.get(&dfa).cloned()) else {
        return false;
    };

    let document = web_sys::window().unwrap().document().unwrap();
    let Some(canvas) = document.get_element_by_id(canvas_id) else {
        return false;
    };

    let canvas: Result<HtmlCanvasElement, _> = canvas.dyn_into();
    let Ok(canvas) = canvas else {
        return false;
    };

    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    let mut drawer = CanvasDrawer::new(context);
    dandy_draw::draw_dfa(&dfa, &mut drawer);

    true
}

#[wasm_bindgen]
pub fn draw_nfa(nfa: usize, canvas_id: &str) -> bool {
    let Some(nfa) = NFA_MAP.with_borrow(|map| map.get(&nfa).cloned()) else {
        return false;
    };

    let document = web_sys::window().unwrap().document().unwrap();
    let Some(canvas) = document.get_element_by_id(canvas_id) else {
        return false;
    };

    let canvas: Result<HtmlCanvasElement, _> = canvas.dyn_into();
    let Ok(canvas) = canvas else {
        return false;
    };

    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    let mut drawer = CanvasDrawer::new(context);
    dandy_draw::draw_nfa(&nfa, &mut drawer);

    true
}

#[wasm_bindgen]
pub fn check_dfa_eq(dfa1: usize, dfa2: usize) -> Option<bool> {
    DFA_MAP.with_borrow(|map| {
        Option::zip(map.get(&dfa1), map.get(&dfa2)).map(|(dfa1, dfa2)| dfa1.equivalent_to(dfa2))
    })
}

#[wasm_bindgen]
pub fn check_nfa_eq(nfa1: usize, nfa2: usize) -> Option<bool> {
    NFA_MAP.with_borrow(|map| {
        Option::zip(map.get(&nfa1), map.get(&nfa2)).map(|(nfa1, nfa2)| nfa1.equivalent_to(nfa2))
    })
}

#[wasm_bindgen]
pub fn dfa_to_nfa(dfa: usize) -> Option<usize> {
    let dfa = DFA_MAP.with_borrow(|map| map.get(&dfa).cloned())?;
    let nfa = dfa.to_nfa();
    Some(push_nfa(nfa))
}

#[wasm_bindgen]
pub fn nfa_to_dfa(nfa: usize) -> Option<usize> {
    let nfa = NFA_MAP.with_borrow(|map| map.get(&nfa).cloned())?;
    let dfa = nfa.to_dfa();
    Some(push_dfa(dfa))
}

#[wasm_bindgen]
pub fn dfa_to_table(dfa: usize) -> Option<String> {
    DFA_MAP.with_borrow(|map| map.get(&dfa).map(Dfa::to_table))
}

#[wasm_bindgen]
pub fn nfa_to_table(nfa: usize) -> Option<String> {
    NFA_MAP.with_borrow(|map| map.get(&nfa).map(Nfa::to_table))
}

#[wasm_bindgen]
pub fn delete_regex(regex: usize) -> bool {
    REGEX_MAP.with_borrow_mut(|map| map.remove(&regex).is_some())
}

#[wasm_bindgen]
pub fn load_regex(input: &str) -> Result<usize, String> {
    let regex: Regex =
        dandy::parser::regex(input).map_err(|e| format!("Error parsing Regex: {e:?}"))?;
    Ok(push_regex(regex))
}

fn push_regex(regex: Regex) -> usize {
    let key = gen_key();
    REGEX_MAP.with_borrow_mut(|map| {
        map.insert(key, regex);
    });
    key
}

#[wasm_bindgen]
pub fn delete_dfa(dfa: usize) -> bool {
    DFA_MAP.with_borrow_mut(|map| map.remove(&dfa).is_some())
}

#[wasm_bindgen]
pub fn load_dfa(input: &str) -> Result<usize, String> {
    let dfa: Dfa = dandy::parser::dfa(input)
        .map_err(|e| format!("Error parsing DFA: {e:?}"))?
        .try_into()
        .map_err(|e: DfaParseError| e.to_string())?;
    Ok(push_dfa(dfa))
}

fn push_dfa(dfa: Dfa) -> usize {
    let key = gen_key();
    DFA_MAP.with_borrow_mut(|map| {
        map.insert(key, dfa);
    });
    key
}

#[wasm_bindgen]
pub fn delete_nfa(nfa: usize) -> bool {
    NFA_MAP.with_borrow_mut(|map| map.remove(&nfa).is_some())
}

#[wasm_bindgen]
pub fn load_nfa(input: &str) -> Result<usize, String> {
    let nfa: Nfa = dandy::parser::nfa(input)
        .map_err(|e| format!("Error parsing NFA: {e:?}"))?
        .try_into()
        .map_err(|e: NfaParseError| e.to_string())?;
    Ok(push_nfa(nfa))
}

fn push_nfa(nfa: Nfa) -> usize {
    let key = gen_key();
    NFA_MAP.with_borrow_mut(|map| {
        map.insert(key, nfa);
    });
    key
}

fn gen_key() -> usize {
    KEYGEN.with_borrow_mut(|gen| gen.next().unwrap())
}
