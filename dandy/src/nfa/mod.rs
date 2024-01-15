use crate::nfa::eval::NfaEvaluator;
use crate::table::Table;
use std::collections::HashSet;
use std::{iter, mem};

pub mod eval;
pub mod parse;

#[derive(Clone, Debug)]
pub struct Nfa {
    alphabet: Vec<String>,
    states: Vec<NfaState>,
    initial_state: usize,
}

#[derive(Clone, Debug)]
pub struct NfaState {
    name: String,
    initial: bool,
    accepting: bool,
    epsilon_transitions: Vec<usize>,
    transitions: Vec<Vec<usize>>,
}

impl Nfa {
    pub fn accepts(&self, string: &[&str]) -> bool {
        let mut eval = self.evaluator();
        eval.step_multiple(string);
        eval.is_accepting()
    }

    pub fn evaluator(&self) -> NfaEvaluator<'_> {
        self.into()
    }

    pub fn closure(&self, start: usize) -> Option<HashSet<usize>> {
        if start >= self.states.len() {
            return None;
        }
        let mut all = HashSet::new();
        all.insert(start);
        let mut new = vec![start];
        while !new.is_empty() {
            let old_new = mem::take(&mut new);
            for state in old_new {
                for &eps_target in &self.states[state].epsilon_transitions {
                    if all.insert(eps_target) {
                        new.push(eps_target)
                    }
                }
            }
        }
        Some(all)
    }

    pub fn to_table(&self) -> String {
        let mut table = Table::default();

        let mut alph = vec!["", "", "", "ε"];
        alph.extend(self.alphabet.iter().map(|s| s as &str));
        table.push_row(alph);

        let trans_strings = &self
            .states
            .iter()
            .map(|state| {
                iter::once(&state.epsilon_transitions)
                    .chain(&state.transitions)
                    .map(|trans| {
                        let s = trans
                            .iter()
                            .map(|c| self.states[*c].name.clone())
                            .collect::<Vec<_>>()
                            .join(" ");
                        format!("{{{s}}}")
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        for (idx, state) in self.states.iter().enumerate() {
            let mut state = vec![
                if state.initial { "->" } else { "" },
                if state.accepting { "*" } else { "" },
                &state.name,
            ];
            state.extend(trans_strings[idx].iter().map(|s| s as &str));
            table.push_row(state);
        }
        table.to_string(" ")
    }

    pub fn equivalent_to(&self, other: &Nfa) -> bool {
        //if the alphabets are different, they aren't equivalent
        if self.alphabet.len() != other.alphabet.len() {
            return false;
        }

        let set1 = self.alphabet.iter().collect::<HashSet<_>>();
        let set2 = other.alphabet.iter().collect::<HashSet<_>>();
        if set1 != set2 {
            return false;
        }

        // initially, we explore the (pair of) initial states
        let mut evaluators_to_explore = vec![(self.evaluator(), other.evaluator())];
        let mut explored_states = HashSet::new();
        explored_states.insert((
            {
                let mut vec = evaluators_to_explore[0]
                    .0
                    .current_states_idx()
                    .iter()
                    .copied()
                    .collect::<Vec<_>>();
                vec.sort();
                vec
            },
            {
                let mut vec = evaluators_to_explore[0]
                    .1
                    .current_states_idx()
                    .iter()
                    .copied()
                    .collect::<Vec<_>>();
                vec.sort();
                vec
            },
        ));

        while let Some((s1, s2)) = evaluators_to_explore.pop() {
            // we explore states s1 and s2
            // they must both be accepting or rejecting
            if s1.is_accepting() != s2.is_accepting() {
                return false;
            }
            // for each char in alphabet, we step the evaluator. If we get new states, explore them!
            for elem in self.alphabet.iter() {
                let mut d1 = s1.clone();
                d1.step(elem);
                let mut d2 = s2.clone();
                d2.step(elem);
                if explored_states.insert((
                    {
                        let mut vec = d1.current_states_idx().iter().copied().collect::<Vec<_>>();
                        vec.sort();
                        vec
                    },
                    {
                        let mut vec = d2.current_states_idx().iter().copied().collect::<Vec<_>>();
                        vec.sort();
                        vec
                    },
                )) {
                    evaluators_to_explore.push((d1, d2));
                }
            }
        }
        true
    }
}
