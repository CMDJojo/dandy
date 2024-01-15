use std::collections::{HashSet};
use std::ops::Index;
use crate::table::Table;

pub mod parse;

#[derive(Clone, Debug)]
pub struct Dfa {
    alphabet: Vec<String>,
    states: Vec<DfaState>,
    initial_state: usize,
}

#[derive(Clone, Debug)]
pub struct DfaState {
    name: String,
    initial: bool,
    accepting: bool,
    transitions: Vec<usize>,
}

impl Dfa {
    pub fn to_table(&self) -> String {
        let mut table = Table::default();

        let mut alph = vec!["", ""];
        alph.extend(self.alphabet.iter().map(|s| s as &str));
        table.push_row(alph);

        for DfaState {
            name,
            initial,
            accepting,
            transitions
        } in &self.states {
            let initial = match (*initial, *accepting) {
                (true, true) => "-> *",
                (true, false) => "->",
                (false, true) => "   *",
                (false, false) => ""
            };
            let mut state = vec![initial, name];
            transitions.iter().for_each(
                |&c| state.push(&self.states[c].name)
            );
            table.push_row(state);
        }
        table.to_string(" ")
    }

    pub fn equivalent_to(&self, other: &Dfa) -> bool {
        //if the alphabets are different, they aren't equivalent
        if self.alphabet.len() != other.alphabet.len() {
            return false;
        }

        //alphabet mapping
        //note: if we find all from 1st in 2nd it is the same due to uniqueness
        let alphabet_map: Vec<_> = self.alphabet
            .iter()
            .filter_map(|c| other.alphabet.iter().position(|x| x == c))
            .collect();

        //if the map didn't manage to map all from self to other, they are different
        if alphabet_map.len() != self.alphabet.len() {
            return false;
        }

        // initially, we explore the (pair of) initial states
        let mut states_to_explore = vec![(
            &self.states[self.initial_state],
            &other.states[other.initial_state]
        )];
        let mut explored_states = HashSet::new();
        explored_states.insert((self.initial_state, other.initial_state));

        while !states_to_explore.is_empty() {
            // we explore states s1 and s2
            let (s1, s2) = states_to_explore.pop().unwrap();
            // they must both be accepting or rejecting
            if s1.accepting != s2.accepting {
                return false;
            }
            // for each char in alphabet, we get a pair of transitions
            // note we need to map the alphabet
            (0..self.alphabet.len()).for_each(|self_alph_idx| {
                let other_alph_idx = alphabet_map[self_alph_idx];
                let self_idx = s1.transitions[self_alph_idx];
                let other_idx = s2.transitions[other_alph_idx];

                if explored_states.insert((self_idx, other_idx)) {
                    let pair = (
                        &self.states[self_idx],
                        &other.states[other_idx]
                    );
                    states_to_explore.push(pair);
                }
            });
        }
        return true;
    }
}