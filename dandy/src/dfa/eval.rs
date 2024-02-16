use crate::dfa::{Dfa, DfaState};
use std::collections::HashMap;
use std::iter;

#[derive(Debug, Clone)]
pub struct DfaEvaluator<'a> {
    dfa: &'a Dfa,
    rev_map: HashMap<&'a str, usize>,
    current_state: usize,
    unknown_elem_seen: bool,
}

impl<'a> DfaEvaluator<'a> {
    pub fn is_accepting(&self) -> bool {
        self.current_state().map_or(false, DfaState::is_accepting)
    }

    pub fn current_state(&self) -> Option<&DfaState> {
        if self.unknown_elem_seen {
            None
        } else {
            Some(&self.dfa.states[self.current_state])
        }
    }

    pub fn current_state_idx(&self) -> usize {
        // FIXME: Option<usize>
        self.current_state
    }

    pub fn step_all(&self) -> Vec<DfaEvaluator<'a>> {
        iter::repeat(self.clone())
            .zip(self.dfa.alphabet())
            .map(|(mut eval, elem)| {
                eval.step(elem);
                eval
            })
            .collect()
    }

    pub fn step(&mut self, elem: &str) -> Option<&DfaState> {
        if self.unknown_elem_seen {
            return None;
        }

        match self.rev_map.get(elem) {
            None => {
                self.unknown_elem_seen = true;
                None
            }
            Some(&idx) => {
                self.current_state = self.dfa.states[self.current_state].transitions[idx];
                Some(&self.dfa.states[self.current_state])
            }
        }
    }

    pub fn step_multiple(&mut self, elems: &[&str]) -> Option<&DfaState> {
        match elems.iter().try_for_each(|e| self.step(e).map(|_| ())) {
            None => {
                self.unknown_elem_seen = true;
                None
            }
            Some(_) => Some(&self.dfa.states[self.current_state]),
        }
    }
}

impl<'a> From<&'a Dfa> for DfaEvaluator<'a> {
    fn from(value: &'a Dfa) -> Self {
        let map = value
            .alphabet
            .iter()
            .enumerate()
            .map(|(idx, c)| (c as &str, idx))
            .collect();
        Self {
            dfa: value,
            rev_map: map,
            current_state: value.initial_state,
            unknown_elem_seen: false,
        }
    }
}
