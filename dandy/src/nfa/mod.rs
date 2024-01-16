use crate::dfa::{Dfa, DfaState};
use crate::nfa::eval::NfaEvaluator;
use crate::table::Table;
use std::collections::{HashMap, HashSet};
use std::{iter, mem};

pub mod eval;
pub mod parse;

#[derive(Clone, Debug)]
pub struct Nfa {
    pub(crate) alphabet: Vec<String>,
    pub(crate) states: Vec<NfaState>,
    pub(crate) initial_state: usize,
}

#[derive(Clone, Debug)]
pub struct NfaState {
    pub(crate) name: String,
    pub(crate) initial: bool,
    pub(crate) accepting: bool,
    pub(crate) epsilon_transitions: Vec<usize>,
    pub(crate) transitions: Vec<Vec<usize>>,
}

impl Nfa {
    pub fn to_dfa(&self) -> Dfa {
        let mut gen = 0usize..;
        let mut map = HashMap::new(); // mapping vec names to numbers
        let mut accepting = HashSet::new();
        let mut to_explore = vec![self.evaluator()];
        let mut transitions = HashMap::new();
        let key = Self::set_to_vec(to_explore[0].current_states_idx());
        let n = gen.next().unwrap();
        map.insert(key, n);
        if to_explore[0].is_accepting() {
            accepting.insert(n);
        }

        while let Some(eval) = to_explore.pop() {
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
            alphabet: self.alphabet.clone(),
            states,
            initial_state: 0,
        }
    }

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

        let mut alph = vec!["", "", "", "ε"];
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
                if state.initial { "->" } else { "" },
                if state.accepting { "*" } else { "" },
                &state.name,
            ];
            state.extend(trans_strings[idx].iter().map(|s| s as &str));
            table.push_row(state);
        }
        table.to_string(" ")
    }

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

    fn set_to_vec(set: &HashSet<usize>) -> Vec<usize> {
        let mut vec = set.iter().copied().collect::<Vec<_>>();
        vec.sort();
        vec
    }
}
