//! # Regular expressions
//! Dandy implements some mathematical definitions of Regular Expressions, which is a subset of the regexes commonly
//! found for pattern matching in programming languages.
//!
//! ## Syntax
//! Regular expressions are written in a UTF-8 encoded file. Each unicode extended grapheme clusters is considered
//! one character (but no normalization is used). Sequencing is done by concatenating characters. There are
//! eight reserved characters: `(`, `)`, `∅`, `ε`, `|`, `*`, `+` and `\`. These needs to be escaped with a backslash
//! (`\`), while all other characters are supported. Parenthesis `(`,`)` is used for grouping, `∅` denotes the empty
//! language, `ε` denotes the empty string, `|` denotes alternation, and `*`/`+` is Kleene star/plus (zero or more/one
//! or more). Initial and trailing whitespace is ignored, but all whitespace within the expression is significant.
//!
//! Here are some examples:
//! * `(ab)+` matches `ab`, `abab`, `ababab`, ...
//! * `(ab)*` matches `(empty string)`, `ab`, `abab`, `ababab`, ...
//! * `0*1(0+ε)` matches `1`, `10`, `0001` and all other strings containing the character `1` once
//!
//! ## Operations
//! The only operation currently implemented is converting a Regular Expression to a NFA. From there, you can do lots
//! of stuff, like optimizing it, encoding it to a table, enumerate all words in it, convert it to a DFA to take the
//! symmetric difference to another regex or automata etc.
//!
//! Here are some example usages of the regexes above:
//! ```
//! use dandy::parser;
//! let regex1 = parser::regex("(ab)+").unwrap();
//! let regex2 = parser::regex("(ab)*").unwrap();
//! let regex3 = parser::regex("0*1(0|ε)").unwrap();
//!
//! let nfa1 = regex1.to_nfa();
//! let nfa2 = regex2.to_nfa();
//! let mut nfa3 = regex3.to_nfa();
//!
//! assert!(&["ab", "abab", "ababab"].iter().all(|s| nfa1.accepts_graphemes(s)));
//! assert!(&["", "ab", "abab", "ababab"].iter().all(|s| nfa2.accepts_graphemes(s)));
//! assert!(&["1", "10", "0001"].iter().all(|s| nfa3.accepts_graphemes(s)));
//!
//! let dfa1 = nfa1.to_dfa();
//! let dfa2 = nfa2.to_dfa();
//! let mut symmetric_difference = dfa1.symmetric_difference(&dfa2).unwrap().to_nfa();
//! // The only word not in both regex1 and regex2 is the empty word
//! let mut words = symmetric_difference.words();
//! assert_eq!(words.next(), Some("".to_string()));
//! assert_eq!(words.next(), None);
//!
//! nfa3.remove_epsilon_moves(); // Note: word enumeration is currently only available for NFAs without epsilon moves
//! let mut words = nfa3.words();
//! // Words are always enumerated lexicographically
//! assert_eq!(words.next(), Some("1".to_string()));
//! assert_eq!(words.next(), Some("01".to_string()));
//! assert_eq!(words.next(), Some("10".to_string()));
//! ```

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

    /// *This is subject to change*
    pub fn to_string(&self) -> String {
        let mut acc = String::new();
        Self::build_string(&self.tree, &mut acc);
        acc
    }

    fn build_string(tree: &RegexTree, acc: &mut String) {
        match tree {
            RegexTree::Sequence(seq) => {
                for item in seq {
                    Self::build_string(item, acc);
                }
            }
            RegexTree::Alt(seq) => {
                acc.push('(');
                let mut iter = seq.iter();
                if let Some(first) = iter.next() {
                    Self::build_string(first, acc);
                    for item in iter {
                        acc.push('|');
                        Self::build_string(item, acc);
                    }
                }
                acc.push(')');
            }
            RegexTree::Repeat(seq) => {
                acc.push('(');
                Self::build_string(seq, acc);
                acc.push(')');
                acc.push('*');
            }
            RegexTree::Char(c) => match c {
                RegexChar::Epsilon => {
                    acc.push('ε');
                }
                RegexChar::Empty => {
                    acc.push('∅');
                }
                RegexChar::Grapheme(g) => {
                    if g.len() == 1
                        && ['(', ')', '∅', 'ε', '|', '*', '+', '\\']
                            .contains(&g.chars().next().unwrap())
                    {
                        acc.push('\\');
                        acc.push_str(g);
                    } else {
                        acc.push_str(g);
                    }
                }
            },
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
