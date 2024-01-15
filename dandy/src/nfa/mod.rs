use std::iter;
use std::ops::Index;
use crate::table::Table;

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