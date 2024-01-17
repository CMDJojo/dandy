use crate::dfa::eval::DfaEvaluator;
use crate::nfa::{Nfa, NfaState};
use crate::table::Table;
use std::collections::HashSet;

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
