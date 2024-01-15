use std::collections::{HashMap, HashSet};
use thiserror::Error;
use crate::nfa::{Nfa, NfaState};
use crate::parser::{NfaAlphabetEntry, ParsedNfa, ParsedNfaState};

#[derive(Debug, Error)]
pub enum NfaParseError<'a> {
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
}

impl<'a> TryFrom<ParsedNfa<'a>> for Nfa {
    type Error = NfaParseError<'a>;

    fn try_from(value: ParsedNfa<'a>) -> Result<Self, Self::Error> {
        use NfaParseError::*;
        let ParsedNfa {
            head, states
        } = value;

        let mut eps_idx = None;
        {
            let mut alphabet = HashSet::new();
            head.iter().enumerate().try_for_each(|(idx, e)| {
                match e {
                    NfaAlphabetEntry::Element(c) => alphabet.insert(c).then_some(()).ok_or(c),
                    NfaAlphabetEntry::Eps => {
                        if eps_idx.is_some() {
                            Err(&"ε")
                        } else {
                            eps_idx = Some(idx);
                            Ok(())
                        }
                    }
                }
            }).map_err(|e| DuplicateAlphabetSymbol(e))?
        }

        let state_name_map: HashMap<_, _> =
            states.iter().enumerate().map(|(i, s)| (s.name, i)).collect();
        let mut initial_state = None;

        let mut new_states = Vec::with_capacity(states.len());
        for (idx, state) in states.into_iter().enumerate() {
            let ParsedNfaState {
                name, initial, accepting, transitions
            } = state;

            if transitions.len() != head.len() {
                return Err(
                    WrongNumberOfTransitions(
                        name,
                        transitions.len(),
                        head.len(),
                    )
                ); // Alphabet and state transitions does not have same len
            }

            let mut epsilon_transitions = None;
            let mut new_transitions = Vec::with_capacity(head.len());
            for (idx, transition) in transitions.iter().enumerate() {
                let mut tr_idx = Vec::with_capacity(transition.len());
                if Some(idx) == eps_idx {
                    for target in transition {
                        if let Some(idx) = state_name_map.get(target) {
                            tr_idx.push(*idx);
                        } else {
                            return Err(
                                TransitionDoesNotExist(
                                    name,
                                    target,
                                )
                            ); // Target of transition does not exist
                        }
                    }
                    epsilon_transitions = Some(tr_idx);
                } else {
                    for target in transition {
                        if let Some(idx) = state_name_map.get(target) {
                            tr_idx.push(*idx);
                        } else {
                            return Err(
                                TransitionDoesNotExist(
                                    name,
                                    target,
                                )
                            ); // Target of transition does not exist
                        }
                    }
                    new_transitions.push(tr_idx);
                }
            }

            if initial {
                if initial_state.is_none() {
                    initial_state = Some(idx);
                } else {
                    return Err(
                        MultipleInitialStates
                    );
                }
            }

            new_states.push(NfaState {
                name: name.to_string(),
                initial,
                accepting,
                epsilon_transitions: epsilon_transitions.unwrap_or_default(),
                transitions: new_transitions,
            });
        }

        if let Some(initial_state) = initial_state {
            let dfa = Nfa {
                alphabet: head.iter().filter_map(|s|
                    match s {
                        NfaAlphabetEntry::Eps => None,
                        NfaAlphabetEntry::Element(s) => Some(s.to_string())
                    }).collect::<Vec<String>>(),
                states: new_states,
                initial_state,
            };
            Ok(dfa)
        } else {
            Err(
                MissingInitialState
            )
        }
    }
}
