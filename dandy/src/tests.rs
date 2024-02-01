use crate::dfa::{Dfa, DfaState};
use crate::nfa::{Nfa, NfaState};
use crate::*;
use ::regex::Regex as LibRegex;
use proptest::prelude::*;
use rand::prelude::*;
use std::collections::HashSet;
use std::ops::RangeInclusive;
use std::rc::Rc;

#[test]
fn test_subset_construction() {
    let dfa_source = include_str!("../tests/test_files/eq_to_nfa1.dfa");
    let parsed_dfa = parser::dfa(dfa_source).unwrap();
    let dfa: dfa::Dfa = parsed_dfa.try_into().unwrap();

    let nfa_source = include_str!("../tests/test_files/nfa1.nfa");
    let parsed_nfa = parser::nfa(nfa_source).unwrap();
    let nfa: nfa::Nfa = parsed_nfa.try_into().unwrap();

    let converted = nfa.to_dfa();
    assert!(dfa.equivalent_to(&converted));
}

proptest! {
    /// Tests that a DFA can be turned into a table with dfa.to_table() and then be
    /// parsed to the *very same* DFA again (not just equivalent)
    #[test]
    fn dfa_table_reparse(dfa in dfa(50, 50)) {
        let parsed_dfa: Dfa = parser::dfa(&dfa.to_table()).unwrap().try_into().unwrap();
        assert_eq!(dfa, parsed_dfa);
    }

    /// Tests that a DFA can be minimized and is then still equivalent to the original DFA
    #[test]
    fn dfa_minimize_eq(dfa in dfa(25, 25)) { // This size is adequate, larger size takes too long time
        let mut minimized_dfa = dfa.clone();
        minimized_dfa.minimize();
        assert!(minimized_dfa.equivalent_to(&dfa), "Minimized DFA should be equivalent to original");
        assert!(dfa.equivalent_to(&minimized_dfa), "Original DFA should be equivalent to original");
    }

    /// Tests that a DFA can be turned into an NFA and then turned back again to a DFA
    /// while still being equivalent to the original DFA
    #[test]
    fn dfa_to_nfa_to_dfa(dfa in dfa(50, 50)) {
        let converted = dfa.clone().to_nfa().to_dfa();
        assert!(dfa.equivalent_to(&converted), "DFA should be equivalent to DFA->NFA->DFA");
        assert!(converted.equivalent_to(&dfa), "DFA->NFA->DFA should be equivalent to DFA");
    }


    /// Tests that a NFA can be turned into a table with dfa.to_table() and then be
    /// parsed to the *very same* DFA again (not just equivalent)
    #[test]
    fn nfa_table_reparse(nfa in nfa(50, 50)) {
        let parsed_nfa: Nfa = parser::nfa(&nfa.to_table()).unwrap().try_into().unwrap();
        assert_eq!(nfa, parsed_nfa);
    }

    /// Tests that a NFA can be turned into an DFA and then turned back again to a NFA
    /// while still being equivalent to the original NFA
    #[test]
    fn nfa_to_dfa_to_nfa(nfa in nfa(25, 25)) {
        let converted = nfa.to_dfa().to_nfa();
        assert!(nfa.equivalent_to(&converted), "NFA should be equivalent to NFA->DFA->NFA");
        assert!(converted.equivalent_to(&nfa), "NFA->DFA->NFA should be equivalent to NFA");
    }

    #[test]
    fn binary_dfa_ops(
        dfa1 in fixed_alphabet_dfa(20, 'a'..='f', ('a'..='f').count()),
        dfa2 in fixed_alphabet_dfa(20, 'a'..='f', ('a'..='f').count()),
        tests in prop::collection::vec("[a-f]+", 100)
    ) {
        let intersection = dfa1.intersection(&dfa2).unwrap();
        let union = dfa1.union(&dfa2).unwrap();
        let difference = dfa1.difference(&dfa2).unwrap();
        let symmetric_difference = dfa1.symmetric_difference(&dfa2).unwrap();
        for test in tests.iter() {
            let r1 = dfa1.accepts_graphemes(test);
            let r2 = dfa2.accepts_graphemes(test);
            assert_eq!(intersection.accepts_graphemes(test), r1 && r2);
            assert_eq!(union.accepts_graphemes(test), r1 || r2);
            assert_eq!(difference.accepts_graphemes(test), r1 && !r2);
            assert_eq!(symmetric_difference.accepts_graphemes(test), r1 != r2);
        }
    }

    #[test]
    fn dfa_self_union(dfa in fixed_alphabet_dfa(20, 'a'..='z', ('a'..='z').count())) {
        let union = dfa.union(&dfa).unwrap();
        assert!(union.equivalent_to(&dfa));
    }

    #[test]
    fn dfa_self_intersection(dfa in fixed_alphabet_dfa(20, 'a'..='z', ('a'..='z').count())) {
        let intersection = dfa.intersection(&dfa).unwrap();
        assert!(intersection.equivalent_to(&dfa));
    }

    #[test]
    fn dfa_inversion_tautologies(
        dfa in fixed_alphabet_dfa(20, 'a'..='f', ('a'..='f').count()),
        tests in prop::collection::vec("[a-f]+", 100)
    ) {
        let inv_dfa = {
            let mut dfa = dfa.clone();
            dfa.invert();
            dfa
        };
        let union = dfa.union(&inv_dfa).unwrap();
        let intersection = dfa.intersection(&inv_dfa).unwrap();
        tests.iter().for_each(|test| {
            assert!(union.accepts_graphemes(test));
            assert!(union.has_reachable_accepting_state());
            assert!(!intersection.accepts_graphemes(test));
            assert!(!intersection.has_reachable_accepting_state());
        });
    }

    #[test]
    fn regex(
        regex_str in random_regex(),
        tests in prop::collection::vec("[a-z]+", 20)
    ) {
        let regex = parser::regex(&regex_str).unwrap();
        let mut dfa = regex.to_nfa().to_dfa();
        dfa.minimize();
        let lib_regex = LibRegex::new(&format!("^({regex_str})$")).unwrap();

        let accepted_chars = regex_str.chars().collect::<HashSet<_>>();

        tests.iter().for_each(|test|{
            // Need to filter string since it can't use characters not in the regex itself
            // due to the DFA alphabet
            let s = test.chars().filter(|c| accepted_chars.contains(c)).collect::<String>();
            assert_eq!(dfa.accepts_graphemes(&s), lib_regex.is_match(&s));
        })
    }
}

prop_compose! {
    fn nfa(max_states: usize, max_alphabet_size: usize)
        (num_states in 1..max_states, alphabet_size in 1..max_alphabet_size)
        (
            states in state_names(num_states),
            alphabet in alphabet_elems(alphabet_size),
            initial_state in 0..num_states,
            accepting_states in prop::collection::vec(any::<bool>(), num_states..=num_states),
            epsilon_transitions in prop::collection::vec(epsilon_transitions(num_states), num_states..=num_states),
            transitions in prop::collection::vec(nfa_transitions(num_states, alphabet_size), num_states..=num_states)
        )
    -> Nfa {
        let states = states.into_iter().zip(
            accepting_states.into_iter().zip(
                transitions.into_iter().zip(
                    epsilon_transitions.into_iter()
                )
            )
        ).enumerate().map(|(idx, (state_name, (accepting, (transitions, epsilon_transitions))))|
            NfaState {
                name: Rc::from(state_name.as_str()),
                initial: idx == initial_state,
                accepting,
                epsilon_transitions,
                transitions
            }
        ).collect();

        Nfa {
            alphabet: alphabet.iter().map(|entry| Rc::from(entry.as_str())).collect(),
            states,
            initial_state
        }
    }
}

prop_compose! {
    fn fixed_alphabet_dfa(max_states: usize, alphabet: RangeInclusive<char>, alphabet_size: usize)
        (num_states in 1..max_states)
        (
            states in state_names(num_states),
            initial_state in 0..num_states,
            accepting_states in prop::collection::vec(any::<bool>(), num_states..=num_states),
            transitions in prop::collection::vec(dfa_transitions(num_states, alphabet_size), num_states..=num_states)
        )
    -> Dfa {
        let states = states.into_iter().zip(
            accepting_states.into_iter().zip(
                transitions.into_iter()
            )
        ).enumerate().map(|(idx, (state_name, (accepting, transitions)))|
            DfaState {
                name: Rc::from(state_name.as_str()),
                initial: idx == initial_state,
                accepting,
                transitions
            }
        ).collect();

        let mut alphabet: Vec<Rc<str>> = alphabet.clone().map(|c| Rc::from(c.to_string())).collect();
        alphabet.shuffle(&mut thread_rng());
        let alphabet = Rc::from(alphabet);

        Dfa {
            alphabet,
            states,
            initial_state
        }
    }
}

prop_compose! {
    fn dfa(max_states: usize, max_alphabet_size: usize)
        (num_states in 1..max_states, alphabet_size in 1..max_alphabet_size)
        (
            states in state_names(num_states),
            alphabet in alphabet_elems(alphabet_size),
            initial_state in 0..num_states,
            accepting_states in prop::collection::vec(any::<bool>(), num_states..=num_states),
            transitions in prop::collection::vec(dfa_transitions(num_states, alphabet_size), num_states..=num_states)
        )
    -> Dfa {
        let states = states.into_iter().zip(
            accepting_states.into_iter().zip(
                transitions.into_iter()
            )
        ).enumerate().map(|(idx, (state_name, (accepting, transitions)))|
            DfaState {
                name: Rc::from(state_name.as_str()),
                initial: idx == initial_state,
                accepting,
                transitions
            }
        ).collect();

        Dfa {
            alphabet: alphabet.iter().map(|entry| Rc::from(entry.as_str())).collect(),
            states,
            initial_state
        }
    }
}

prop_compose! {
    fn dfa_transitions(states: usize, alphabet_size: usize)
        (transitions in prop::collection::vec(0..states, alphabet_size..=alphabet_size))
    -> Vec<usize> {
        transitions
    }
}

prop_compose! {
    fn epsilon_transitions(states: usize)
        (transitions in prop::collection::vec(any::<bool>(), states..=states))
    -> Vec<usize> {
        let mut rng = thread_rng();
        let mut transitions: Vec<_> = transitions.into_iter()
            .enumerate()
            .filter_map(|(idx, b)| b.then_some(idx))
            .collect();
        transitions.shuffle(&mut rng);
        transitions
    }
}

prop_compose! {
    fn nfa_transitions(states: usize, alphabet_size: usize)
        (transitions in prop::collection::vec(
            // This is a bytevec saying for each state if it has a transition there or not
            // HashMap would be a better fit but maybe too much rejections?
            prop::collection::vec(any::<bool>(), states..=states),
            alphabet_size..=alphabet_size
        ))
    -> Vec<Vec<usize>> {
        let mut rng = thread_rng();
        transitions.into_iter()
            .map(|row| {
                let mut row: Vec<usize> = row.into_iter()
                    .enumerate()
                    .filter_map(|(idx, b)| b.then_some(idx))
                    .collect();
                row.as_mut_slice().shuffle(&mut rng);
                row
            })
            .collect()
    }
}

prop_compose! {
    fn state_names(count: usize)
        (names in filtered_set(count, r"[^\s#{}]+", &["ε", "eps", "→", "->", "*"]))
    -> HashSet<String> {
        names
    }
}

prop_compose! {
    fn simple_alphabet(count: usize)
        (names in filtered_set(std::cmp::max(count, 4), "[a-e]", &[]))
    -> HashSet<String> {
        names
    }
}

prop_compose! {
    fn alphabet_elems(count: usize)
        (names in filtered_set(count, r"[^\s#{}]+", &["ε", "eps", "→", "->", "*"]))
    -> HashSet<String> {
        names
    }
}

prop_compose! {
    fn filtered_set(count: usize, regex: &'static str, deny: &'static [&'static str])
        (names in prop::collection::hash_set(
            regex.prop_filter( // No whitespace
                "name should not be reserved",
                |s| !deny.contains(&s.as_str()) && !s.contains(|c: char| c.is_whitespace())
            ),
            count..=count
        ))
    -> HashSet<String> {
        names
    }
}

fn random_regex() -> impl Strategy<Value = String> {
    "[a-z]".prop_recursive(20, 1024, 20, |inner| {
        prop_oneof![
            10 => prop::collection::vec(inner.clone(), 1..20)
                .prop_map(|vec| format!("({})", vec.join(""))),
            10 => prop::collection::vec(inner.clone(), 1..20).prop_map(|vec| vec.join("|")),
            3 => inner.clone().prop_map(|r| format!("({r})*")),
            3 => inner.clone().prop_map(|r| format!("({r})+")),
        ]
    })
}
