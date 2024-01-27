use crate::nfa::{Nfa, NfaState};
use std::collections::HashMap;
use std::iter;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Regex {
    pub tree: RegexTree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexTree {
    Sequence(Vec<RegexTree>),
    Alt(Vec<RegexTree>),
    Repeat(Box<RegexTree>),
    Char(RegexChar),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexChar {
    Grapheme(Rc<str>),
    Epsilon,
    Empty,
}

#[derive(Clone, Debug)]
struct StateCounter {
    state: usize,
}

impl StateCounter {
    fn new() -> Self {
        Self { state: 0 }
    }

    fn next(&mut self) -> usize {
        let old = self.state;
        self.state += 1;
        old
    }

    fn peek(&self) -> usize {
        self.state
    }
}

impl Regex {
    /// Converts this regular expression to a NFA. This is the only operation available to regular expressions.
    /// To check if a string is accepted by this regular expression, one should convert it to a NFA and then check
    /// using that NFA. Note that the resulting NFA may be quite large, so converting it to a DFA may optimize it.
    pub fn to_nfa(self) -> Nfa {
        // Final accepting state is 0
        // Initial state is 1
        let mut counter = StateCounter::new();

        let mut char_map = HashMap::new();
        let mut idx_acc = 0..;
        let mut grapheme_idx =
            |g: Rc<str>| -> usize { *char_map.entry(g).or_insert_with(|| idx_acc.next().unwrap()) };

        let accepting_state = NfaState {
            name: Rc::from(counter.next().to_string()),
            initial: false,
            accepting: true,
            epsilon_transitions: vec![],
            transitions: vec![],
        };

        // The initial state should send to the first thing in the tree
        let initial_state = NfaState {
            name: Rc::from(counter.next().to_string()),
            initial: true,
            accepting: false,
            epsilon_transitions: vec![counter.peek()],
            transitions: vec![],
        };

        let states = {
            let mut tree_states = Self::tree_to_nfa(self.tree, &mut counter, &mut grapheme_idx, 0);
            let mut all_states = Vec::with_capacity(tree_states.len() + 2);
            all_states.push(accepting_state); // state 0
            all_states.push(initial_state); // state 1
            all_states.append(&mut tree_states);
            // need to extend all transition tables to alphabet length
            all_states
                .iter_mut()
                .for_each(|s| s.transitions.resize(char_map.len(), vec![]));
            all_states
        };

        let alphabet = {
            let mut sorted_map = char_map.into_iter().collect::<Vec<_>>();
            sorted_map.sort_by_key(|(_, i)| *i);
            sorted_map.into_iter().map(|(s, _)| s).collect()
        };

        Nfa {
            alphabet,
            states,
            initial_state: 1,
        }
    }

    /// We turn a tree to a NFA recursively. `counter` is used to get the number of the next state.
    /// `char_idx` gives the index of a given character in the alphabet (and inserts the character
    /// if it didn't exist already). `send_to` is the state that the subtree should transition to
    /// if successful.
    fn tree_to_nfa(
        tree: RegexTree,
        counter: &mut StateCounter,
        grapheme_idx: &mut impl FnMut(Rc<str>) -> usize,
        send_to: usize,
    ) -> Vec<NfaState> {
        let incoming_state_idx = counter.next();
        let mut incoming_state = NfaState {
            name: Rc::from(incoming_state_idx.to_string()),
            initial: false,
            accepting: false,
            epsilon_transitions: vec![],
            transitions: vec![],
        };

        match tree {
            RegexTree::Sequence(seq) => {
                if seq.is_empty() {
                    incoming_state.epsilon_transitions.push(send_to);
                    vec![incoming_state]
                } else {
                    incoming_state.epsilon_transitions.push(counter.state + 1);
                    let seq_len = seq.len();
                    let mut states = seq
                        .into_iter()
                        .enumerate()
                        .flat_map(|(idx, subtree)| {
                            let after_state_idx = counter.next();
                            let mut after_state = NfaState {
                                name: Rc::from(after_state_idx.to_string()),
                                initial: false,
                                accepting: false,
                                epsilon_transitions: vec![],
                                transitions: vec![],
                            };
                            let new_states =
                                Self::tree_to_nfa(subtree, counter, grapheme_idx, after_state_idx);
                            if idx + 1 == seq_len {
                                after_state.epsilon_transitions.push(send_to);
                            } else {
                                after_state.epsilon_transitions.push(counter.state + 1);
                            }
                            iter::once(after_state).chain(new_states)
                        })
                        .collect::<Vec<_>>();
                    let mut ret = vec![incoming_state];
                    ret.append(&mut states);
                    ret
                }
            }
            RegexTree::Alt(alt) => {
                let mut additional = alt
                    .into_iter()
                    .flat_map(|tree| {
                        incoming_state.epsilon_transitions.push(counter.peek());
                        Self::tree_to_nfa(tree, counter, grapheme_idx, send_to)
                    })
                    .collect::<Vec<_>>();
                let mut ret = Vec::with_capacity(1 + additional.len());
                ret.push(incoming_state);
                ret.append(&mut additional);
                ret
            }
            RegexTree::Repeat(r) => {
                incoming_state.epsilon_transitions = vec![counter.peek(), send_to];
                let mut additional =
                    Self::tree_to_nfa(*r, counter, grapheme_idx, incoming_state_idx);
                let mut ret = Vec::with_capacity(additional.len() + 1);
                ret.push(incoming_state);
                ret.append(&mut additional);
                ret
            }
            RegexTree::Char(c) => match c {
                RegexChar::Grapheme(g) => {
                    // If we only accept one char, make sure our incoming state
                    // transition to outgoing state on that char only
                    let cidx = grapheme_idx(g); // our character index

                    // if we get index 1, we want {{}, {target}} in our transition table
                    let mut transition_vec = vec![vec![]; cidx];
                    transition_vec.push(vec![send_to]);
                    incoming_state.transitions = transition_vec;
                    vec![incoming_state]
                }
                RegexChar::Epsilon => {
                    // If we accept epsilon char, just transition to send to immediately
                    incoming_state.epsilon_transitions = vec![send_to];
                    vec![incoming_state]
                }
                RegexChar::Empty => {
                    vec![incoming_state]
                }
            },
        }
    }
}
