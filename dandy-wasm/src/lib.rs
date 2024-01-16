use std::cell::RefCell;
use dandy::dfa::parse::DfaParseError;
use dandy::dfa::Dfa;
use wasm_bindgen::prelude::wasm_bindgen;
use dandy::nfa::Nfa;
use dandy::nfa::parse::NfaParseError;

thread_local! {
    static DFA_LIST: RefCell<Vec<Dfa>> = RefCell::default();
    static NFA_LIST: RefCell<Vec<Nfa>> = RefCell::default();
}

#[wasm_bindgen]
pub fn check_dfa_eq(dfa1: usize, dfa2: usize) -> Option<bool> {
    DFA_LIST.with_borrow(|list|
        Option::zip(
            list.get(dfa1),
            list.get(dfa2),
        ).map(|(dfa1, dfa2)| dfa1.equivalent_to(dfa2))
    )
}

#[wasm_bindgen]
pub fn check_nfa_eq(nfa1: usize, nfa2: usize) -> Option<bool> {
    NFA_LIST.with_borrow(|list|
        Option::zip(
            list.get(nfa1),
            list.get(nfa2),
        ).map(|(nfa1, nfa2)| nfa1.equivalent_to(nfa2))
    )
}

#[wasm_bindgen]
pub fn dfa_to_nfa(dfa: usize) -> Option<usize> {
    let dfa = DFA_LIST.with_borrow(|list| list.get(dfa).cloned())?;
    let nfa = dfa.to_nfa();
    Some(push_nfa(nfa))
}

#[wasm_bindgen]
pub fn nfa_to_dfa(nfa: usize) -> Option<usize> {
    let nfa = NFA_LIST.with_borrow(|list| list.get(nfa).cloned())?;
    let dfa = nfa.to_dfa();
    Some(push_dfa(dfa))
}

#[wasm_bindgen]
pub fn dfa_to_table(dfa: usize) -> Option<String> {
    DFA_LIST.with_borrow(|list|
        list.get(dfa).map(Dfa::to_table)
    )
}

#[wasm_bindgen]
pub fn nfa_to_table(nfa: usize) -> Option<String> {
    NFA_LIST.with_borrow(|list|
        list.get(nfa).map(Nfa::to_table)
    )
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
    DFA_LIST.with_borrow_mut(|list| {
        list.push(dfa);
        list.len() - 1
    })
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
    NFA_LIST.with_borrow_mut(|list| {
        list.push(nfa);
        list.len() - 1
    })
}

#[wasm_bindgen]
pub fn check_eq(a: &str, b: &str) -> String {
    let parse1 = dandy::parser::dfa(a);
    if let Err(err) = parse1 {
        return format!("Error parsing 1: {}", err);
    }
    let p_dfa1 = parse1.unwrap();
    let dfa1: Result<Dfa, DfaParseError> = p_dfa1.try_into();
    if let Err(err) = dfa1 {
        return format!("Error compiling 1: {}", err);
    }
    let dfa1: Dfa = dfa1.unwrap();

    let parse2 = dandy::parser::dfa(b);
    if let Err(err) = parse2 {
        return format!("Error parsing 2: {}", err);
    }
    let p_dfa2 = parse2.unwrap();
    let dfa2: Result<Dfa, DfaParseError> = p_dfa2.try_into();
    if let Err(err) = dfa2 {
        return format!("Error compiling 2: {}", err);
    }
    let dfa2: Dfa = dfa2.unwrap();

    if dfa1.equivalent_to(&dfa2) {
        "Equivalent".to_string()
    } else {
        "Not equivalent".to_string()
    }
}
