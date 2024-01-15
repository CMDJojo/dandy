use crate::dfa::eval::DfaEvaluator;
use crate::table::Table;
use std::collections::HashSet;
use std::ops::Index;

pub mod eval;
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
    pub fn accepts(&self, string: &[&str]) -> bool {
        let mut eval = self.evaluator();
        eval.step_multiple(string);
        eval.is_accepting()
    }

    pub fn evaluator(&self) -> DfaEvaluator<'_> {
        self.into()
    }

    pub fn to_table(&self) -> String {
        let mut table = Table::default();

        let mut alph = vec!["", "", ""];
        alph.extend(self.alphabet.iter().map(|s| s as &str));
        table.push_row(alph);

        for DfaState {
            name,
            initial,
            accepting,
            transitions,
        } in &self.states
        {
            let mut state = vec![
                if *initial { "->" } else { "" },
                if *accepting { "*" } else { "" },
                name,
            ];
            transitions
                .iter()
                .for_each(|&c| state.push(&self.states[c].name));
            table.push_row(state);
        }
        table.to_string(" ")
    }

    pub fn equivalent_to(&self, other: &Dfa) -> bool {
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
            evaluators_to_explore[0].0.current_state_idx(),
            evaluators_to_explore[0].1.current_state_idx(),
        ));

        while !evaluators_to_explore.is_empty() {
            // we explore states s1 and s2
            let (s1, s2) = evaluators_to_explore.pop().unwrap();
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
                if explored_states.insert((d1.current_state_idx(), d2.current_state_idx())) {
                    evaluators_to_explore.push((d1, d2));
                }
            }
        }
        return true;
    }
}
