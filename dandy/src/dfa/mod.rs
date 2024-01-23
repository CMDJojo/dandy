use crate::dfa::eval::DfaEvaluator;
use crate::nfa::{Nfa, NfaState};
use crate::table::Table;
use std::collections::{HashMap, HashSet};

pub mod eval;
pub mod parse;

/// A deterministic finite automata, denoted by its alphabet, states and the initial state
#[derive(Clone, Debug)]
pub struct Dfa {
    pub(crate) alphabet: Vec<String>,
    pub(crate) states: Vec<DfaState>,
    pub(crate) initial_state: usize,
}

/// A state in a DFA automata, which consists of its name, if it is the initial state or not, if it is accepting
/// or not, and the transition for each element of the alphabet
#[derive(Clone, Debug)]
pub struct DfaState {
    pub(crate) name: String,
    pub(crate) initial: bool,
    pub(crate) accepting: bool,
    pub(crate) transitions: Vec<usize>,
}

impl DfaState {
    /// Gets the name of this state
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Checks if this state is the initial state
    pub fn is_initial(&self) -> bool {
        self.initial
    }

    /// Checks if this state is accepting
    pub fn is_accepting(&self) -> bool {
        self.accepting
    }

    /// Gets a list of transitions, as state indices for each element of the alphabet, in the alphabet's ordering
    pub fn transitions(&self) -> &[usize] {
        self.transitions.as_slice()
    }
}

impl From<DfaState> for NfaState {
    fn from(value: DfaState) -> Self {
        let DfaState {
            name,
            initial,
            accepting,
            transitions,
        } = value;
        NfaState {
            name,
            initial,
            accepting,
            epsilon_transitions: vec![],
            transitions: transitions.into_iter().map(|t| vec![t]).collect(),
        }
    }
}

impl From<Dfa> for Nfa {
    fn from(value: Dfa) -> Self {
        value.to_nfa()
    }
}

impl Dfa {
    /// Minimizes this DFA by first removing all unreachable states and then merging non-distinguishable states
    pub fn minimize(&mut self) {
        self.remove_unreachable_states();
        self.merge_nondistinguishable_states();
    }

    /// Merges the non-distinguishable states of this DFA such that every set of multiple non-distinguishable states
    /// become just one. Which of multiple non-distinguishable states is left over is non-deterministic
    pub fn merge_nondistinguishable_states(&mut self) {
        let mapper = self
            .state_equivalence_classes_idx()
            .into_iter()
            .flat_map(|set| {
                debug_assert!(!set.is_empty(), "Should not have empty equivalence classes");
                let mut iter = set.into_iter();
                let new = iter.next();
                // safety: iter.map is lazy, so body is only executed if there are elements,
                // which means new must be non-optional
                iter.map(move |old| (old, unsafe { new.unwrap_unchecked() }))
            })
            .collect::<HashMap<_, _>>();
        let map = |idx| mapper.get(&idx).copied();
        self.remap_transitions(map);
        let to_remove = mapper.into_iter().map(|(old, _)| old).collect();
        self.remove_states(to_remove);
    }

    /// Gives the equivalence classes of the states of this DFA, which is the sets of non-distinguishable states
    pub fn state_equivalence_classes(&self) -> Vec<Vec<&DfaState>> {
        self.state_equivalence_classes_idx()
            .into_iter()
            .map(|class| {
                class
                    .into_iter()
                    .map(|state| &self.states[state])
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Gives the equivalence classes of the states of this DFA, which is the sets of non-distinguishable states, by
    /// their indices
    pub fn state_equivalence_classes_idx(&self) -> Vec<HashSet<usize>> {
        let (finals, nonfinals) =
            (0..self.states.len()).partition(|&idx| self.states[idx].accepting);
        let mut p: Vec<HashSet<_>> = vec![finals, nonfinals];
        let mut w = p.clone();

        // Hopcroft's algorithm
        while let Some(a) = w.pop() {
            for c in 0..self.alphabet.len() {
                let x: HashSet<usize> = self
                    .states
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| a.contains(&s.transitions[c]))
                    .map(|(i, _)| i)
                    .collect();
                p = p
                    .into_iter()
                    .map(|y| {
                        (
                            x.intersection(&y).copied().collect::<HashSet<_>>(),
                            y.difference(&x).copied().collect::<HashSet<_>>(),
                            y,
                        )
                    })
                    .flat_map(|(inters, diff, y)| {
                        if !inters.is_empty() && !diff.is_empty() {
                            if let Some(idx) = w.iter().position(|hs| hs == &y) {
                                w.swap_remove(idx);
                                w.push(inters.clone());
                                w.push(diff.clone());
                            } else if inters.len() <= diff.len() {
                                w.push(inters.clone());
                            } else {
                                w.push(diff.clone());
                            }
                            // ugly to allocate vec but fck monomorphism and static dispatch
                            // wont work with slices or iter::once or smth
                            vec![inters, diff].into_iter()
                        } else {
                            vec![y].into_iter()
                        }
                    })
                    .collect()
            }
        }
        p
    }

    /// Removes the unreachable states of this automata
    pub fn remove_unreachable_states(&mut self) {
        let states = self.unreachable_state_idx().into_iter().collect();
        self.remove_states(states);
    }

    /// Finds the unreachable states, that is, all states that cannot be reached by any input to the automata
    pub fn unreachable_states(&self) -> Vec<&DfaState> {
        self.unreachable_state_idx()
            .into_iter()
            .map(|idx| &self.states[idx])
            .collect()
    }

    /// Finds the unreachable states, that is, all states that cannot be reached by any input to the automata, and
    /// returns them as indices
    pub fn unreachable_state_idx(&self) -> HashSet<usize> {
        let mut unreachables = (0..self.states.len()).collect::<HashSet<_>>();
        unreachables.remove(&self.initial_state);
        let mut new_states = HashSet::from([self.initial_state]);
        while !new_states.is_empty() {
            new_states = new_states
                .drain()
                .flat_map(|state| self.states[state].transitions.iter().copied())
                .filter(|state| unreachables.remove(state))
                .collect();
        }
        unreachables
    }

    /// Remaps the transitions so that any transition to n gets mapped to mapper(n) (if any, otherwise n is preserved)
    fn remap_transitions(&mut self, mapper: impl Fn(usize) -> Option<usize>) {
        self.states.iter_mut().for_each(|state| {
            state
                .transitions
                .iter_mut()
                .for_each(|trans| *trans = mapper(*trans).unwrap_or(*trans))
        })
    }

    /// This function removes the states with indices in the vector from this DFA, changing the transition tables
    /// of the remaining states to the new state indices. There should not be any transitions to any of the states
    /// that are to be removed. If there is, transitions may be undefined after this call. If debug_assertions is
    /// enabled, such errors would cause a panic here, otherwise they would not be and the DFA might panic at a later
    /// stage
    fn remove_states(&mut self, mut to_remove: Vec<usize>) {
        let mut old_state_idx = (0..self.states.len()).collect::<Vec<_>>();

        to_remove.sort();
        to_remove.iter().rev().for_each(|&idx| {
            self.states.remove(idx);
            old_state_idx.remove(idx);
        });

        let map = |idx| {
            let res = old_state_idx.binary_search(&idx);
            if cfg!(debug_assertions) {
                Some(res.expect("No transitions to removed state"))
            } else {
                res.ok()
            }
        };
        self.remap_transitions(map);
    }

    /// Converts this DFA to a NFA by simply converting each state to a NFA state. All state names
    /// are kept. This is a cheap operation, involving no clones but some vector allocations due to
    /// the vectors required by NFA.
    pub fn to_nfa(self) -> Nfa {
        let Dfa {
            alphabet,
            states,
            initial_state,
        } = self;
        let states = states.into_iter().map(|s| s.into()).collect();
        Nfa {
            alphabet,
            states,
            initial_state,
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
    pub fn evaluator(&self) -> DfaEvaluator<'_> {
        self.into()
    }

    /// Generates a table of this DFA suitable for printing, which may be parsed again to this automaton
    pub fn to_table(&self) -> String {
        self.gen_table("→")
    }

    /// Generates a table of this DFA suitable for printing, which may be parsed again to this automaton. The arrow for
    /// the initial state is "->"
    pub fn ascii_table(&self) -> String {
        self.gen_table("->")
    }

    fn gen_table(&self, arrow: &str) -> String {
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
                if *initial { arrow } else { "" },
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

    /// Checks if this DFA is equivalent to another DFA, that is, if they accept the same language.
    /// If the automatons have different alphabets they are never equivalent, but the order of the alphabet,
    /// the number of states and the transitions doesn't matter.
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
                if explored_states.insert((d1.current_state_idx(), d2.current_state_idx())) {
                    evaluators_to_explore.push((d1, d2));
                }
            }
        }
        true
    }

    /// Gets the alphabet of this DFA
    pub fn alphabet(&self) -> &[String] {
        self.alphabet.as_slice()
    }

    /// Gets the states of this DFA
    pub fn states(&self) -> &[DfaState] {
        self.states.as_slice()
    }

    /// Gets the initial state of this DFA
    pub fn initial_state(&self) -> &DfaState {
        &self.states[self.initial_state]
    }

    /// Get the index of the initial state of this DFA
    pub fn initial_state_index(&self) -> usize {
        self.initial_state
    }
}
