use std::collections::{HashSet};
use std::ops::Index;

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
        let widest_state =
            self.states
                .iter()
                .map(|s| s.name.chars().count())
                .max()
                .unwrap_or_default();

        let widest_col =
            self.alphabet.iter()
                .map(|s| s.chars().count())
                .max()
                .unwrap_or_default()
                .max(widest_state);

        let pad = |s: &str, l: usize| {
            let cs = s.chars().count();
            if cs < l {
                let amnt = l - cs;
                format!("{}{}", s, &" ".repeat(amnt))
            } else {
                s.to_string()
            }
        };

        let mut lines = Vec::with_capacity(self.states.len() + 1);
        let alphabet = self.alphabet.iter().map(
            |c| pad(c, widest_col)
        ).collect::<Vec<_>>().join(" ");
        //                        -> * A
        lines.push(format!("     {} {alphabet}", " ".repeat(widest_state)));

        for DfaState {
            name,
            initial,
            accepting,
            transitions
        } in &self.states {
            let init = if *initial { "->" } else { "  " };
            let accept = if *accepting { "*" } else { " " };
            let name= pad(name, widest_state);
            let trans = transitions.iter().map(
                |&c| pad(&self.states[c].name, widest_col)
            ).collect::<Vec<_>>().join(" ");
            lines.push(format!("{init} {accept} {name} {trans}"));
        }
        lines.join("\n")
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