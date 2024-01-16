use crate::dfa::{Dfa, DfaState};
use crate::nfa::eval::NfaEvaluator;
use crate::table::Table;
use std::collections::{HashMap, HashSet};
use std::{iter, mem};

pub mod eval;
pub mod parse;

/// A non-determenistic finite automata, denoted by its alphabet, states and the initial state
#[derive(Clone, Debug)]
pub struct Nfa {
    pub(crate) alphabet: Vec<String>,
    pub(crate) states: Vec<NfaState>,
    pub(crate) initial_state: usize,
}

/// A state in a NFA automata, which consists of its name, if it is the initial state or not, if it is accepting
/// or not, any amount of epsilon transitions and any amount of transitions for each element in alphabet
#[derive(Clone, Debug)]
pub struct NfaState {
    pub(crate) name: String,
    pub(crate) initial: bool,
    pub(crate) accepting: bool,
    pub(crate) epsilon_transitions: Vec<usize>,
    pub(crate) transitions: Vec<Vec<usize>>,
}

impl Nfa {
    /// Converts this NFA to a DFA using the powerset construction.
    /// Note that this is a somewhat expensive operation. The names of
    /// the states in the resulting DFA are non-deterministic, named
    /// sequentially from 0. The state named 0 is guaranteed to be the
    /// initial state
    pub fn to_dfa(&self) -> Dfa {
        // Generator to generate sequential numbers to new states
        let mut gen = 0usize..;
        // Mapping set of old states to new sequential number
        let mut map = HashMap::new();
        // Set of sequential numbers which are accepting states
        let mut accepting = HashSet::new();
        // Evaluators to explore
        let mut to_explore = vec![self.evaluator()];
        // Transition tables for new states, indexed by sets
        let mut transitions = HashMap::new();

        {
            // Pre-work, add init to tables
            let key = Self::set_to_vec(to_explore[0].current_states_idx());
            let n = gen.next().unwrap(); // 0
            map.insert(key, n);
            if to_explore[0].is_accepting() {
                accepting.insert(n);
            }
        }

        // While we have non-expanded states
        while let Some(eval) = to_explore.pop() {
            // Keep track of transitions from this state
            let mut tr = Vec::with_capacity(self.alphabet.len());
            for new_evaluator in eval.step_all() {
                let is_accepting = new_evaluator.is_accepting();
                let key = Self::set_to_vec(new_evaluator.current_states_idx());
                if !map.contains_key(&key) {
                    to_explore.push(new_evaluator);
                }
                let x = map.entry(key).or_insert_with(|| gen.next().unwrap());
                tr.push(*x);
                if is_accepting {
                    accepting.insert(*x);
                }
            }

            transitions.insert(Self::set_to_vec(eval.current_states_idx()), tr);
        }

        let sorted_keys = {
            let mut vec = map.iter().collect::<Vec<_>>();
            vec.sort_by_key(|(_, &n)| n);
            vec
        };

        // We sort the keys to have a nice table later on. This may be wasteful but
        // Self::set_to_vec sorts and converts sets to vecs anyways so nevermind
        let states = sorted_keys
            .into_iter()
            .map(|(key, &n)| DfaState {
                name: n.to_string(),
                initial: n == 0,
                accepting: accepting.contains(&n),
                transitions: transitions.remove(key).unwrap(),
            })
            .collect();

        Dfa {
            alphabet: self.alphabet.clone(), // Prob only clone we can avoid if we take borrow
            states,
            initial_state: 0, // We start at initial state and assign 0 from gen, so initial is 0
        }
    }

    /// Checks if this automaton accepts the given string. This is equivalent to getting the
    /// evaluator, stepping it multiple times and checking if it is accepting
    pub fn accepts(&self, string: &[&str]) -> bool {
        let mut eval = self.evaluator();
        eval.step_multiple(string);
        eval.is_accepting()
    }

    /// Gets an evaluator, which is a struct that is used to evaluate strings with the automaton
    pub fn evaluator(&self) -> NfaEvaluator<'_> {
        self.into()
    }

    /// Gives the epsilon closure of a state, given the state index
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

    /// Generates a table of this NFA suitable for printing, which may be parsed again to this automaton
    pub fn to_table(&self) -> String {
        self.gen_table("ε", "→")
    }

    /// Generates a table of this NFA suitable for printing, which may be parsed again to this automaton. The epsilon
    /// character is represented "eps" and the arrow for the initial state is "->"
    pub fn ascii_table(&self) -> String {
        self.gen_table("eps", "->")
    }

    fn gen_table(&self, eps: &str, arrow: &str) -> String {
        let mut table = Table::default();

        let mut alph = vec!["", "", "", eps];
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
                if state.initial { arrow } else { "" },
                if state.accepting { "*" } else { "" },
                &state.name,
            ];
            state.extend(trans_strings[idx].iter().map(|s| s as &str));
            table.push_row(state);
        }
        table.to_string(" ")
    }

    /// Checks if this NFA is equivalent to another NFA, that is, if they accept the same language.
    /// If the automatons have different alphabets they are never equivalent, but the order of the alphabet,
    /// the number of states and the transitions doesn't matter.
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
            Self::set_to_vec(evaluators_to_explore[0].0.current_states_idx()),
            Self::set_to_vec(evaluators_to_explore[0].1.current_states_idx()),
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
                    Self::set_to_vec(d1.current_states_idx()),
                    Self::set_to_vec(d2.current_states_idx()),
                )) {
                    evaluators_to_explore.push((d1, d2));
                }
            }
        }
        true
    }

    /// Converts a HashSet (which is not hashable) to a Vec (which is hashable) in a determenistic way
    fn set_to_vec<T: Clone + Ord>(set: &HashSet<T>) -> Vec<T> {
        let mut vec = set.iter().cloned().collect::<Vec<_>>();
        vec.sort();
        vec
    }
}
