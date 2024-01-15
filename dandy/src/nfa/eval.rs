use crate::nfa::{Nfa, NfaState};
use std::collections::{HashMap, HashSet};
use std::iter;

#[derive(Clone, Debug)]
pub struct NfaEvaluator<'a> {
    nfa: &'a Nfa,
    rev_map: HashMap<&'a str, usize>,
    current_states: HashSet<usize>,
}

impl<'a> NfaEvaluator<'a> {
    pub fn is_accepting(&self) -> bool {
        self.current_states().iter().any(|s| s.accepting)
    }

    pub fn current_states(&self) -> Vec<&NfaState> {
        self.current_states
            .iter()
            .map(|&s| &self.nfa.states[s])
            .collect()
    }

    pub fn current_states_idx(&self) -> &HashSet<usize> {
        &self.current_states
    }

    pub fn step_all(&self) -> Vec<NfaEvaluator> {
        iter::repeat(self.clone())
            .zip(&self.nfa.alphabet)
            .map(|(mut eval, elem)| {
                eval.step(elem);
                eval
            })
            .collect()
    }

    pub fn step(&mut self, elem: &str) -> Option<()> {
        let &idx = self.rev_map.get(elem)?;
        self.current_states = self
            .current_states
            .iter()
            .flat_map(|&state| self.nfa.states[state].transitions[idx].clone())
            .collect();
        self.include_closure();
        Some(())
    }

    pub fn step_multiple(&mut self, elems: &[&str]) -> Option<()> {
        elems.iter().try_for_each(|e| self.step(e))
    }

    fn include_closure(&mut self) {
        let mut updated = true;
        let mut to_push = HashSet::new();
        while updated {
            updated = false;
            for state in self.current_states.iter() {
                for epsilon_state in self.nfa.states[*state].epsilon_transitions.iter() {
                    if !self.current_states.contains(epsilon_state) {
                        updated = true;
                        to_push.insert(epsilon_state);
                    }
                }
            }
            self.current_states.extend(to_push.drain());
        }
    }
}

impl<'a> From<&'a Nfa> for NfaEvaluator<'a> {
    fn from(value: &'a Nfa) -> Self {
        let map = value
            .alphabet
            .iter()
            .enumerate()
            .map(|(idx, c)| (c as &str, idx))
            .collect();
        let mut evaluator = Self {
            nfa: value,
            rev_map: map,
            current_states: HashSet::new(),
        };
        evaluator.current_states.insert(value.initial_state);
        evaluator.include_closure();
        evaluator
    }
}
