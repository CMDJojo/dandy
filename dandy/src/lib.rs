pub mod dfa;
pub mod nfa;
pub mod parser;
mod table;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subset_construction() {
        let dfa_source = include_str!("../tests/test_files/eq_to_nfa1.dfa");
        let parsed_dfa = parser::full_dfa(dfa_source).unwrap().1;
        let dfa: dfa::Dfa = parsed_dfa.try_into().unwrap();

        let nfa_source = include_str!("../tests/test_files/nfa1.nfa");
        let parsed_nfa = parser::full_nfa(nfa_source).unwrap().1;
        let nfa: nfa::Nfa = parsed_nfa.try_into().unwrap();

        let converted = nfa.to_dfa();
        assert!(dfa.equivalent_to(&converted));
    }
}
