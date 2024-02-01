//! # Deterministic finite automaton
//! The DFA module includes the [Dfa] struct which represents a
//! [Deterministic finite automaton](https://en.wikipedia.org/wiki/Deterministic_finite_automaton). Currently,
//! the only two ways to create such an instance is by [converting a NFA to a DFA](Nfa::to_dfa) or by parsing from a
//! string.
//!
//! ## Parsing
//! You may parse a state transition table in text form to a DFA. The parsing is done in two steps, the first one
//! just parsing into a [ParsedDfa](crate::parser::ParsedDfa) and the second one checking the invariant of that
//! parsed DFA and converting it into a [Dfa]:
//! ```
//! use dandy::dfa::parse::DfaParseError;
//! use crate::dandy::dfa::{Dfa, parse};
//!
//! fn parse() {
//!     // A DFA with initial state s1, two accepting states s2 and s4,
//!     // accepting all strings with an odd number of a:s
//!     let input = "
//!                a  b
//!         ->  s1 s2 s1
//!           * s2 s3 s1
//!             s3 s4 s1
//!           * s4 s1 s1
//!     ";
//!     // Parsing the DFA
//!     let parsed_dfa = dandy::parser::dfa(input).unwrap();
//!     // Checking invariants
//!     let mut dfa: Dfa = parsed_dfa.try_into().unwrap();
//!     assert!(dfa.accepts(&["a", "b", "b"]));  // odd number of a:s
//!     assert!(!dfa.accepts(&["a", "a", "b"])); // even number of a:s
//!
//!     // We see that states s1 and s3 are non-distinguishable, and that states s2 and s4 are as well.
//!     // Minimizing this DFA will thus result in a DFA with two states
//!     dfa.minimize();
//!     assert_eq!(dfa.states().len(), 2);
//!
//!     let dfa_without_initial_state = "
//!             a b
//!         * x y x
//!           y x y
//!     ";
//!     // A DFA must have an initial state (but it doesn't have to have any accepting states),
//!     // so the invariant should not pass
//!     let parsed_dfa = dandy::parser::dfa(dfa_without_initial_state).unwrap();
//!     let validation: Result<Dfa, DfaParseError<'_>> = parsed_dfa.try_into();
//!     assert_eq!(validation.unwrap_err(), DfaParseError::MissingInitialState);
//! }
//! ```
use crate::dfa::eval::DfaEvaluator;
use crate::nfa::{Nfa, NfaState};
use crate::table::Table;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

pub mod eval;
pub mod parse;

/// A deterministic finite automata, denoted by its alphabet, states and the initial state
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dfa {
    pub(crate) alphabet: Rc<[Rc<str>]>,
    pub(crate) states: Vec<DfaState>,
    pub(crate) initial_state: usize,
}

/// A state in a DFA automata, which consists of its name, if it is the initial state or not, if it is accepting
/// or not, and the transition for each element of the alphabet
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DfaState {
    pub(crate) name: Rc<str>,
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
    /// Inverts this automata, which means that any states that were previously accepting becomes non-accepting and any
    /// states that were previously non-accepting becomes accepting.
    pub fn invert(&mut self) {
        self.states
            .iter_mut()
            .for_each(|s| s.accepting = !s.accepting)
    }

    /// Constructs the union of two DFAs, that is, a new DFA that accepts exactly those strings that are accepted by
    /// the first, second or both DFAs. This returns `None` if and only if the alphabets of the two DFAs are unequal
    /// (not considering ordering).
    ///
    /// ```
    /// use dandy::parser;
    /// use dandy::dfa::Dfa;
    ///
    /// let ends_with_a = "
    ///      a b c
    /// -> n y n n
    ///  * y y n n";
    /// let starts_with_b = "
    ///      a b c
    /// -> i n y n
    ///    n n n n
    ///  * y y y y";
    /// let ends_with_a: Dfa = parser::dfa(ends_with_a).unwrap().try_into().unwrap();
    /// let starts_with_b: Dfa = parser::dfa(starts_with_b).unwrap().try_into().unwrap();
    ///
    /// // 'any' accepts strings that ends with a or starts with b.
    /// let any = ends_with_a.union(&starts_with_b).unwrap();
    /// assert!(any.accepts_graphemes("aa"));
    /// assert!(!any.accepts_graphemes("ab"));
    /// assert!(any.accepts_graphemes("ba"));
    /// assert!(any.accepts_graphemes("bb"));
    /// ```
    pub fn union(&self, other: &Self) -> Option<Self> {
        self.product_construction(other, |s1, s2| s1.accepting || s2.accepting)
    }

    /// Constructs the intersection of two DFAs, that is, a new DFA that accepts exactly those strings that are accepted
    /// by both the first and second DFAs. This returns `None` if and only if the alphabets of the two DFAs are unequal
    /// (not considering ordering).
    ///
    /// ```
    /// use dandy::parser;
    /// use dandy::dfa::Dfa;
    ///
    /// let ends_with_a = "
    ///      a b c
    /// -> n y n n
    ///  * y y n n";
    /// let starts_with_b = "
    ///      a b c
    /// -> i n y n
    ///    n n n n
    ///  * y y y y";
    /// let ends_with_a: Dfa = parser::dfa(ends_with_a).unwrap().try_into().unwrap();
    /// let starts_with_b: Dfa = parser::dfa(starts_with_b).unwrap().try_into().unwrap();
    ///
    /// // 'both' accepts strings that ends with a and starts with b.
    /// let both = ends_with_a.intersection(&starts_with_b).unwrap();
    /// assert!(!both.accepts_graphemes("aa"));
    /// assert!(!both.accepts_graphemes("ab"));
    /// assert!(both.accepts_graphemes("ba"));
    /// assert!(!both.accepts_graphemes("bb"));
    /// ```
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        self.product_construction(other, |s1, s2| s1.accepting && s2.accepting)
    }

    /// Constructs the difference of two DFAs, that is, a new DFA that accepts exactly those strings that are accepted
    /// by the first DFA but not by the second DFA. This returns `None` if and only if the alphabets of the two DFAs are
    /// unequal (not considering ordering).
    ///
    /// ```
    /// use dandy::parser;
    /// use dandy::dfa::Dfa;
    ///
    /// let ends_with_a = "
    ///      a b c
    /// -> n y n n
    ///  * y y n n";
    /// let starts_with_b = "
    ///      a b c
    /// -> i n y n
    ///    n n n n
    ///  * y y y y";
    /// let ends_with_a: Dfa = parser::dfa(ends_with_a).unwrap().try_into().unwrap();
    /// let starts_with_b: Dfa = parser::dfa(starts_with_b).unwrap().try_into().unwrap();
    ///
    /// // 'a_not_b' accepts strings that ends with a and doesn't start with b.
    /// let a_not_b = ends_with_a.difference(&starts_with_b).unwrap();
    /// assert!(a_not_b.accepts_graphemes("aa"));
    /// assert!(!a_not_b.accepts_graphemes("ab"));
    /// assert!(!a_not_b.accepts_graphemes("ba"));
    /// assert!(!a_not_b.accepts_graphemes("bb"));
    /// ```
    pub fn difference(&self, other: &Self) -> Option<Self> {
        self.product_construction(other, |s1, s2| s1.accepting && !s2.accepting)
    }

    /// Constructs the symmetric difference of two DFAs, that is, a new DFA that accepts exactly those strings that are
    /// accepted by either the first or second DFA but not by them both. This returns `None` if and only if the
    /// alphabets of the two DFAs are unequal (not considering ordering).
    ///
    /// ```
    /// use dandy::parser;
    /// use dandy::dfa::Dfa;
    ///
    /// let ends_with_a = "
    ///      a b c
    /// -> n y n n
    ///  * y y n n";
    /// let starts_with_b = "
    ///      a b c
    /// -> i n y n
    ///    n n n n
    ///  * y y y y";
    /// let ends_with_a: Dfa = parser::dfa(ends_with_a).unwrap().try_into().unwrap();
    /// let starts_with_b: Dfa = parser::dfa(starts_with_b).unwrap().try_into().unwrap();
    ///
    /// // 'a_or_b' accepts strings that ends with a or starts with b, but not both.
    /// let a_or_b = ends_with_a.symmetric_difference(&starts_with_b).unwrap();
    /// assert!(a_or_b.accepts_graphemes("aa"));
    /// assert!(!a_or_b.accepts_graphemes("ab"));
    /// assert!(!a_or_b.accepts_graphemes("ba"));
    /// assert!(a_or_b.accepts_graphemes("bb"));
    /// ```
    pub fn symmetric_difference(&self, other: &Self) -> Option<Self> {
        self.product_construction(other, |s1, s2| s1.accepting != s2.accepting)
    }

    /// Constructs a new DFA from two DFAs using the product construction. That is a new DFA with states corresponding
    /// to both the state the first DFA and the second DFA would be in on any given input. If that state is an accepting
    /// state or not is given by the `combinator` function, combining the state from the first parser and the second
    /// parser. `self.product_construction(other, |s1, s2| s1.is_accepting() && s2.is_accepting())` corresponds to
    /// the intersection between the two.
    pub fn product_construction(
        &self,
        other: &Self,
        mut combinator: impl FnMut(&DfaState, &DfaState) -> bool,
    ) -> Option<Self> {
        //if the alphabets are different, they aren't equivalent
        if self.alphabet.len() != other.alphabet.len() {
            return None;
        }

        let set1 = self.alphabet.iter().collect::<HashSet<_>>();
        let set2 = other.alphabet.iter().collect::<HashSet<_>>();
        if set1 != set2 {
            return None;
        }

        // initially, we explore the (pair of) initial states
        let mut evaluators_to_explore = vec![(self.evaluator(), other.evaluator())];
        // initial state pair
        let q1 = self.initial_state;
        let q2 = other.initial_state;
        let mut explored_states = HashSet::new();
        explored_states.insert((q1, q2));

        // maps (q1, q2) to accepting?
        let mut state_data = vec![];

        while let Some((s1, s2)) = evaluators_to_explore.pop() {
            let mut transition_list = Vec::with_capacity(self.alphabet.len());
            for elem in self.alphabet.iter() {
                let mut d1 = s1.clone();
                d1.step(elem);
                let mut d2 = s2.clone();
                d2.step(elem);
                let states = (d1.current_state_idx(), d2.current_state_idx());
                transition_list.push(states);
                if explored_states.insert(states) {
                    evaluators_to_explore.push((d1, d2));
                }
            }

            state_data.push((
                (s1.current_state_idx(), s2.current_state_idx()),
                combinator(s1.current_state(), s2.current_state()),
                transition_list,
            ));
        }

        // Try to generate new names for states
        let names = {
            let mut hm = HashSet::new();
            let potential_names = explored_states
                .iter()
                .map_while(|(s1, s2)| {
                    let combined_name: Rc<str> = Rc::from(format!(
                        "({},{})",
                        self.states[*s1].name, other.states[*s2].name
                    ));
                    hm.insert(combined_name.clone())
                        .then_some(((*s1, *s2), combined_name))
                })
                .collect::<HashMap<_, _>>();
            if potential_names.len() < state_data.len() {
                explored_states
                    .iter()
                    .enumerate()
                    .map(|(idx, (s1, s2))| ((*s1, *s2), Rc::from(format!("{idx}"))))
                    .collect()
            } else {
                potential_names
            }
        };

        let rev_state_idx_map = state_data
            .iter()
            .enumerate()
            .map(|(idx, ((s1, s2), _, _))| ((*s1, *s2), idx))
            .collect::<HashMap<_, _>>();
        let initial_state = *rev_state_idx_map
            .get(&(q1, q2))
            .expect("Initial state should have an index");

        let states = state_data
            .into_iter()
            .map(|(states, accepting, transitions)| DfaState {
                name: names
                    .get(&states)
                    .expect("All states should have a name")
                    .clone(),
                initial: states == (q1, q2),
                accepting,
                transitions: transitions
                    .into_iter()
                    .map(|states| {
                        *rev_state_idx_map
                            .get(&states)
                            .expect("Each state pair with transition to it should have a idx")
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();
        Some(Dfa {
            alphabet: self.alphabet.clone(),
            states,
            initial_state,
        })
    }

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
        if let Some(new_initial) = map(self.initial_state) {
            self.initial_state = new_initial;
        }
        let to_remove = mapper.into_keys().collect();
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
        let (finals, nonfinals): (HashSet<usize>, HashSet<usize>) =
            (0..self.states.len()).partition(|&idx| self.states[idx].accepting);
        if finals.is_empty() {
            return vec![nonfinals];
        } else if nonfinals.is_empty() {
            return vec![finals];
        }
        let mut p = vec![finals, nonfinals];
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
        let reachables = self.reachable_state_idx();
        (0..self.states.len())
            .filter(|x| !reachables.contains(x))
            .collect()
    }

    /// Checks if this DFA has an accepting state that is reachable from the initial state, that is, if it has some
    /// input which it accepts
    pub fn has_reachable_accepting_state(&self) -> bool {
        // Use _idx to not allocate Vec
        self.reachable_state_idx()
            .iter()
            .any(|idx| self.states[*idx].accepting)
    }

    /// Finds the reachable states, that is, all states that can be reached by some input to the automata
    pub fn reachable_states(&self) -> Vec<&DfaState> {
        self.reachable_state_idx()
            .into_iter()
            .map(|idx| &self.states[idx])
            .collect()
    }

    /// Finds the reachable states, that is, all states that can be reached by some input to the automata, and
    /// returns them as indices
    pub fn reachable_state_idx(&self) -> HashSet<usize> {
        let mut reachables = HashSet::from([self.initial_state]);
        let mut new_states = reachables.clone();
        while !new_states.is_empty() {
            new_states = new_states
                .drain()
                .flat_map(|state| self.states[state].transitions.iter().copied())
                .filter(|&state| reachables.insert(state))
                .collect();
        }
        reachables
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
    /// that are to be removed (except for in any of the states that are to be removed). If there is, transitions may be
    /// undefined after this call. If debug_assertions is enabled, such errors would cause a panic here, otherwise they
    /// would not immediately panic but other operations might panic at a later stage. The initial state cannot be
    /// removed and will cause a panic if attempted to.
    fn remove_states(&mut self, mut to_remove: Vec<usize>) {
        let mut old_state_idx = (0..self.states.len()).collect::<Vec<_>>();

        to_remove.sort();
        if let Err(less_than) = to_remove.binary_search(&self.initial_state) {
            // We removed "less than" states before the initial state: adjust
            self.initial_state -= less_than;
        } else {
            panic!("Cannot remove initial state");
        }

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

    /// Checks if this automaton accepts the given string of graphemes, if every grapheme by
    /// itself is considered as an element of the alphabet. Note that if the alphabet contains
    /// elements with multiple graphemes, those won't be recognized. To check if there are
    /// elements with multiple graphemes, see [Dfa::graphemes_only]. A grapheme is defined to be
    /// one extended unicode grapheme cluster (which may consist of one or many code points).
    pub fn accepts_graphemes(&self, string: &str) -> bool {
        let graphemes = string.graphemes(true).collect::<Vec<_>>();
        let mut eval = self.evaluator();
        eval.step_multiple(&graphemes);
        eval.is_accepting()
    }

    /// Checks if the alphabet of this automaton consists of only single graphemes. If it does, one may use
    /// [Dfa::accepts_graphemes] instead of [Dfa::accepts] for improved ergonomics. A grapheme is defined to be
    /// one extended unicode grapheme cluster (which may consist of one or many code points).
    pub fn graphemes_only(&self) -> bool {
        self.alphabet
            .iter()
            .all(|str| str.graphemes(true).count() == 1)
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
    // We could check intersection between one DFA and second DFA complement, and check if it is 0
    // but that would lead to a slowdown of 3964%, so we keep it as is
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
    pub fn alphabet(&self) -> &[Rc<str>] {
        &self.alphabet
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
