use dandy::dfa::parse::DfaParseError;
use dandy::dfa::Dfa;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn check_eq(a: &str, b: &str) -> String {
    let parse1 = dandy::parser::dfa(a);
    if let Err(err) = parse1 {
        return format!("Error parsing 1: {}", err);
    }
    let p_dfa1 = parse1.unwrap().1;
    let dfa1: Result<Dfa, DfaParseError> = p_dfa1.try_into();
    if let Err(err) = dfa1 {
        return format!("Error compiling 1: {}", err);
    }
    let dfa1: Dfa = dfa1.unwrap();

    let parse2 = dandy::parser::dfa(b);
    if let Err(err) = parse2 {
        return format!("Error parsing 2: {}", err);
    }
    let p_dfa2 = parse2.unwrap().1;
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
