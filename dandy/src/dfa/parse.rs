use crate::dfa::{Dfa, DfaState};
use crate::parser::{ParsedDfa, ParsedDfaState};
use std::collections::{HashMap, HashSet};
use std::ops::Not;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DfaParseError<'a> {
    #[error("Wrong number of transitions for state '{0}': has {1} expected {2}")]
    WrongNumberOfTransitions(&'a str, usize, usize),
    #[error("State '{1}' does not exist (in transition from state '{0}')")]
    TransitionDoesNotExist(&'a str, &'a str),
    #[error("There is no initial state")]
    MissingInitialState,
    #[error("There are two (or more) initial states")]
    MultipleInitialStates,
    #[error("'{0}' appears twice in the alphabet")]
    DuplicateAlphabetSymbol(&'a str),
    #[error("State '{0}' defined multiple times")]
    DuplicateStateDefinition(&'a str),
}

impl<'a> TryFrom<ParsedDfa<'a>> for Dfa {
    type Error = DfaParseError<'a>;

    fn try_from(value: ParsedDfa<'a>) -> Result<Self, Self::Error> {
        use DfaParseError::*;
        let ParsedDfa { head, states } = value;

        {
            let mut alphabet = HashSet::new();
            head.iter()
                .try_for_each(|c| alphabet.insert(c).then_some(()).ok_or(c))
                .map_err(|d| DuplicateAlphabetSymbol(d))?;
        }

        let state_name_map: HashMap<_, _> = states
            .iter()
            .enumerate()
            .map(|(i, s)| (s.name, i))
            .collect();

        if state_name_map.len() != states.len() {
            // We have a duplicate name, let's find it!
            let mut seen = HashSet::new();
            let duplicate = states
                .iter()
                .find_map(|s| seen.insert(s.name).not().then_some(s.name))
                .unwrap_or("<unknown>");
            return Err(DuplicateStateDefinition(duplicate));
        }

        let mut initial_state = None;

        let mut new_states = Vec::with_capacity(states.len());
        for (idx, state) in states.into_iter().enumerate() {
            let ParsedDfaState {
                name,
                initial,
                accepting,
                transitions,
            } = state;

            if transitions.len() != head.len() {
                return Err(WrongNumberOfTransitions(
                    name,
                    transitions.len(),
                    head.len(),
                )); // Alphabet and state transitions does not have same len
            }

            let mut new_transitions = Vec::with_capacity(head.len());
            for transition in transitions {
                if let Some(idx) = state_name_map.get(transition) {
                    new_transitions.push(*idx);
                } else {
                    return Err(TransitionDoesNotExist(name, transition)); // Target of transition does not exist
                }
            }

            if initial {
                if initial_state.is_none() {
                    initial_state = Some(idx);
                } else {
                    return Err(MultipleInitialStates);
                }
            }

            new_states.push(DfaState {
                name: Rc::from(name),
                initial,
                accepting,
                transitions: new_transitions,
            });
        }

        if let Some(initial_state) = initial_state {
            let dfa = Dfa {
                alphabet: head.into_iter().map(Rc::from).collect(),
                states: new_states,
                initial_state,
            };
            Ok(dfa)
        } else {
            Err(MissingInitialState)
        }
    }
}
