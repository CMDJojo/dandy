use std::{iter, mem};
use std::collections::HashSet;
use crate::nfa::eval::NfaEvaluator;
use crate::table::Table;

pub mod parse;
pub mod eval;

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

        let mut alph = vec!["", "", "ε"];
        alph.extend(self.alphabet.iter().map(|s| s as &str));
        table.push_row(alph);

        let trans_strings = &self.states.iter().map(
            |state| {
                iter::once(&state.epsilon_transitions)
                    .chain(&state.transitions)
                    .map(|trans| {
                        let s = trans.iter()
                            .map(|c| self.states[*c].name.clone())
                            .collect::<Vec<_>>().join(" ");
                        format!("{{{s}}}")
                    })
                    .collect::<Vec<_>>()
            }
        ).collect::<Vec<_>>();

        for (idx, state) in self.states.iter().enumerate() {
            let initial = match (state.initial, state.accepting) {
                (true, true) => "-> *",
                (true, false) => "->",
                (false, true) => "   *",
                (false, false) => ""
            };
            let mut state = vec![initial, &state.name];
            state.extend(trans_strings[idx].iter().map(|s| s as &str));
            table.push_row(state);
        }
        table.to_string(" ")
    }
}