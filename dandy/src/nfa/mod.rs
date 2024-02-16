//! # Nondeterministic Finite Automaton, with our without ε-moves
//! The NFA module includes the [Nfa] struct which represents a
//! [Nondeterministic finite automaton](https://en.wikipedia.org/wiki/Nondeterministic_finite_automaton) with or without
//! ε-moves. Currently, the only ways to create such an instance is by converting a [Dfa](Dfa::to_nfa) or
//! [Regex](crate::regex::Regex::to_nfa) to a NFA or by parsing from a string.
//!
//! ## Example
//! You may parse a state transition table in text form to a NFA. The parsing is done in two steps, the first one
//! parsing into a [ParsedNfa](crate::parser::ParsedNfa) and the second one checking the invariant of that
//! parsed NFA and converting it into a [Nfa] with [TryInto]:
//! ```
//! use dandy::nfa::parse::NfaParseError;
//! use crate::dandy::nfa::{Nfa, parse};
//!
//! // A NFA without ε-moves with initial state s1, one accepting
//! // state s4, which accepts all strings ending with "aab"
//! let input = "
//!             a       b
//!     ->  s1 {s1 s2} {s1}
//!         s2 {s3}    {}
//!         s3 {}      {s4}
//!       * s4 {}      {}
//! ";
//! // Parsing the NFA
//! let parsed_nfa = dandy::parser::nfa(input).unwrap();
//! // Checking invariants
//! let mut nfa: Nfa = parsed_nfa.try_into().unwrap();
//! assert!(nfa.accepts_graphemes("abaab"));  // ends with "aab"
//! assert!(!nfa.accepts_graphemes("aabb")); // doesn't end with "aab
//!
//! // We know that the first word it accepts is "aab",
//! // and the second is "aaab" followed by "baab"
//! let mut iter = nfa.words();
//! assert_eq!(&iter.next().unwrap(), "aab");
//! assert_eq!(&iter.next().unwrap(), "aaab");
//! assert_eq!(&iter.next().unwrap(), "baab");
//!
//! let nfa_without_initial_state = "
//!          a   b
//!     * x {y} {x}
//!       y {x} {y}
//! ";
//! // A NFA must have an initial state (but it doesn't have to have any accepting states),
//! // so the invariant should not pass
//! let parsed_nfa = dandy::parser::nfa(nfa_without_initial_state).unwrap();
//! let validation: Result<Nfa, NfaParseError<'_>> = parsed_nfa.try_into();
//! assert_eq!(validation.unwrap_err(), NfaParseError::MissingInitialState);
//!
//! ```
//!
//! ## Syntax
//! The file format for NFAs is an UTF-8 encoded text file with more or less just a transition table.
//! The first row of the file should contain all elements of the non-empty alphabet, space-separated. Then,
//! there should be one row per state in the NFA (there must be at least one state), where each row contains
//! these space-separated elements, in order:
//! * Optionally `->` or `→`, if the state is the initial state
//! * Optionally `*`, if the state is accepting
//! * The name of the state (which may not contain whitespace)
//! * For each element of the alphabet specified in the header, in order, what states the Nfa transitions to from the
//!   given state upon seeing that element, as a space-separated set encased in `{` and `}`
//!
//! `ε`, `eps`, `→`, `->` and `*` are reserved and may not be used as elements of the alphabet or names of
//! states.
//!
//! Additionally, these rules apply:
//! * There must be exactly one (1) initial state
//! * All elements of the alphabet should be specified exactly once
//! * Unicode normalization isn't used
//! * All transitions should exist (from every state for every element of the alphabet,
//!   there should be a transition to a set of states where all states are defined. The set may be empty)
//! * To add ε-moves, 'ε' or 'eps' should be added an an element of the alphabet, and the ε-moves should
//!   be entered as if they occurred upon seeing the element 'ε'
//! * Comments are started by '#', and that character and the rest of the line is not parsed
//! * Lines just containing whitespace or comments are ignored
//!
//! ## Operations
//! ### Checking word acceptance
//! The most basic operation to do is to check if a list of elements is accepted by the automata or not.
//! This is done by the [Nfa::accepts] function. Note that there is no restriction to how long an element of
//! the alphabet may be. This means that the [Nfa::accepts] function takes a list of elements (i.e. a list of `&str`'s).
//! Take the following example:
//!
//! ```text
//!        a    aa
//! -> s1 {s1} {s2}
//!  * s2 {s1} {s1}
//! ```
//!
//! If we would be given the input string "aaa", it is ambiguous how to break it down. However, if the alphabet of the
//! NFA consists only of elements which are one single unicode grapheme cluster each (which can be checked by
//! [Nfa::graphemes_only]), then the convenience function [Nfa::accepts_graphemes] can be used to take a `&str` and
//! split it into single grapheme clusters in an unambiguous way before checking. Note that one unicode grapheme cluster
//! may consist of multiple `char`s.
//!
//! Internally, a [NfaEvaluator] is constructed, which is a structure keeping track on the current state during the
//! evaluation of a string. To create a [NfaEvaluator] to use it directly, see [Nfa::evaluator]. One can also check if
//! it is possible to reach an accepting state with [Nfa::has_reachable_accepting_state].
//!
//! Example:
//! ```
//! use dandy::parser;
//! use dandy::nfa::Nfa;
//!
//! // This NFA accepts all strings ending in "aab"
//! let input = "
//!             a       b
//!     ->  s1 {s1 s2} {s1}
//!         s2 {s3}    {}
//!         s3 {}      {s4}
//!       * s4 {}      {}
//! ";
//! let nfa: Nfa = parser::nfa(input).unwrap().try_into().unwrap();
//! // The alphabet of this NFA contains single graphemes only
//! assert!(nfa.graphemes_only());
//! // We can assert that 'aaab' ends in 'aab' in this way...
//! assert!(nfa.accepts(&["a", "a", "a", "b"]));
//! // or since "aaab" becomes "a", "a", "a", "b" when split into graphemes,
//! // we can do
//! assert!(nfa.accepts_graphemes("aaab"));
//! // We can also use the Evaluator manually:
//! let mut evaluator = nfa.evaluator();
//! // We step on "a", "a", "a", and "b"
//! evaluator.step("a");
//! evaluator.step("a");
//! evaluator.step("a");
//! evaluator.step("b");
//! // We should be accepting this input
//! assert!(evaluator.is_accepting());
//! ```
//!
//! ### Conversions
//! We can convert the NFA to a DFA using [Nfa::to_dfa]. This uses a reduced
//! [powerset construction](https://en.wikipedia.org/wiki/Powerset_construction) (or subset
//! construction). Every state of the resulting DFA corresponds to a combination of the states in the NFA. Since each
//! state of the DFA can either include or exclude each state of the NFA, there are a total of `2^n` states in the
//! powerset construction (where `n` is the number of states in the NFA). The reduced powerset construction only
//! includes states that are actually reachable, so most likely not all `2^n` states will be included, but regardless
//! this construction grows exponentially and may lead to very large DFAs.
//!
//! Internally, the alphabet isn't cloned but all new states get new names which are allocated. These details does
//! however have very little performance impact compared to the already inefficient powerset construction which, as
//! said, grows exponentially on the states of the NFA.
//!
//! Unlike for DFAs, there doesn't exist one unique minimized NFA which accepts the language of another NFA.
//! Furthermore, no polynomial time algorithms are known (and none exists under the assumption `P != PSPACE`). This
//! version of Dandy doesn't include a NFA minimization algorithm, but one may be added in the future.
//!
//! The [Nfa] struct represents both NFAs with and without ε-moves, and one could check if the NFA has ε-moves by
//! the [Nfa::has_epsilon_moves] method (which simply loops through the states and checks if any of them has any
//! ε-transitions). The function [Nfa::remove_epsilon_moves] removes all epsilon moves from the NFA by merging the
//! epsilon closure into every normal transition and then clearing the epsilon transitions from each state. This
//! also performs a slight optimization in the sense that all states which only had epsilon transitions gets removed.
//! After a call to [Nfa::remove_epsilon_moves], [Nfa::has_epsilon_moves] will return `false`.
//!
//! In contrast to for DFAs, making all non-accepting states accepting and all accepting states non-accepting doesn't
//! make the NFA accept the complement language. Thus, the [Dfa::invert] function doesn't make much sense for a NFA and
//! isn't added here.
//!
//! Some operations such as calculating difference and symmetric difference is only available on DFAs, and to use them,
//! one should convert the NFA to a DFA first.
//!
//! ### Combining NFAs
//! There are two main ways to combine NFAs: computing their [union](Nfa::union) and their
//! [intersection](Nfa::intersection). For DFAs, there is a useful construction called the product construction, and
//! that exists for NFAs as well, however it isn't as useful. Since a NFA which are in multiple states at once while
//! evaluating a string will accept the string if any of the states it is in is an accepting state, one cannot simply
//! not achieve negation solely by pure product construction. If one would use the product construction to try to
//! calculate the difference or symmetric difference, the product construction will generate a NFA which accepts any
//! string whereupon the first NFA has *any* accepting state and the second NFA has *any* rejecting state reachable
//! on any given string. When calculating the difference, we would need to test if the second NFA is *only* in rejecting
//! states for a given input.
//!
//! Regardless, Dandy provides the product construction for NFAs though the [Nfa::product_construction] function.
//! Moreover, [Nfa::intersection] is implemented by it. [Nfa::union], however, is simply implemented by adding a new
//! initial state with epsilon transitions to both of the NFA:s initial states. Here is an example:
//! ```
//! use dandy::parser;
//! use dandy::nfa::Nfa;
//!
//! let ends_with_aab = "
//!             a       b
//!     ->  s1 {s1 s2} {s1}
//!         s2 {s3}    {}
//!         s3 {}      {s4}
//!       * s4 {}      {}
//! ";
//! let contains_babb = "
//!             a    b
//!     ->  s1 {s1} {s1 s2}
//!         s2 {s3} {}
//!         s3 {}   {s4}
//!         s4 {}   {s5}
//!       * s5 {s5} {s5}
//! ";
//! let ends_with_aab: Nfa = parser::nfa(ends_with_aab).unwrap().try_into().unwrap();
//! let contains_babb: Nfa = parser::nfa(contains_babb).unwrap().try_into().unwrap();
//!
//! // 'any' accepts strings that contains "babb" or ends with "aab".
//! let any = ends_with_aab.union(contains_babb).unwrap();
//! assert!(!any.accepts_graphemes("abbabab"));
//! assert!(any.accepts_graphemes("aaab"));
//! assert!(any.accepts_graphemes("bbabbaab"));
//! assert!(any.accepts_graphemes("bbaabaab"));
//! ```
//!
//! ### Checking equivalence
//! Two NFAs `A` and `B` are equivalent if and only if they have the same alphabet and accept the same language.
//! As discussed earlier, there is no simple way of getting a NFA for the complement language of another NFA, nor can
//! the product construction be used to find the symmetrical difference. Therefore, those ways of checking equivalence
//! of two NFAs are infeasible. Dandy provides [Nfa::equivalent_to] which tracks all states reachable at the same time
//! of the two provided NFAs, and if one is accepting while the other one is not, it rejects the NFAs as not equivalent.
//!
//! ### Enumerating words
//! An algorithm by [Margareta Ackerman and Jeffrey Shallit](https://maya-ackerman.com/wp-content/uploads/2018/09/Enumeration_AckermanShallit2.pdf)
//! for enumerating words in the language of a NFA is implemented in Dandy. The word enumeration has two important
//! features:
//! * The enumeration is done in lexicographic order (according to the order defined in the alphabet)
//! * The enumeration doesn't skip any accepted word (including the empty word)
//!
//! There are three entry points for the word enumeration algorithm, all with the same characteristics but different
//! views of the output. This is due to not having any restrictions on the length of the elements of the alphabet,
//! so as described above, having an alphabet of "a" and "aa", one could not take a generated `String` and split it
//! into the elements of the alphabet. Having said that, here are the three iterators:
//! * [Nfa::words] with `impl Iterator<Item=String>`,
//! * [Nfa::word_components] with `impl Iterator<Item=Vec<Rc<str>>>` where each `Rc<str>` is an element of the alphabet,
//!   and
//! * [Nfa::word_component_indices] with `impl Iterator<Item=Vec<usize>>` where each `usize` is the index to the
//!   element in the alphabet
//!
//! Due to the enumeration not skipping any words, iterating the words of a NFA and its complement will yield all
//! strings of the alphabet.
//!
//! ### Additional operations
//! In addition to the above-mentioned operations, you can:
//! * [Get the alphabet](Nfa::alphabet) of the NFA,
//! * [Get the states](Nfa::states) and [initial state](Nfa::initial_state) of the NFA,
//! * [Convert it to a table](Nfa::to_table), possibly [in ascii-only](Nfa::ascii_table), both of which
//!   can be parsed by Dandy into this very same NFA again,
//! * Find all [reachable](Nfa::reachable_states) and [non-reachable](Nfa::unreachable_states) states,
//! * [Clone](Nfa::clone) it, which isn't super expensive since the alphabet and state names doesn't need new
//!   allocations to be cloned (no strings at all are actually copied, just some `vec`s with `bool`s and `usize`s).
//!   Note that since NFAs can have multiple transitions upon seeing each symbol, cloning a NFA inherently clones more
//!   `vec`s and is more expensive than cloning a DFA.

use crate::dfa::{Dfa, DfaState};
use crate::nfa::words::{WordComponentIndices, WordComponents, Words};
use crate::table::Table;
use crate::util::alphabet_equal;
pub use eval::NfaEvaluator;
pub use parse::NfaParseError;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{iter, mem};
use unicode_segmentation::UnicodeSegmentation;

pub mod eval;
pub mod parse;
pub mod words;

/// A non-deterministic finite automata, denoted by its alphabet, states and the initial state
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nfa {
    pub(crate) alphabet: Rc<[Rc<str>]>,
    pub(crate) states: Vec<NfaState>,
    pub(crate) initial_state: usize,
}

/// A state in a NFA automata, which consists of its name, if it is the initial state or not, if it is accepting
/// or not, any amount of epsilon transitions and any amount of transitions for each element in alphabet
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NfaState {
    pub(crate) name: Rc<str>,
    pub(crate) initial: bool,
    pub(crate) accepting: bool,
    pub(crate) epsilon_transitions: Vec<usize>,
    pub(crate) transitions: Vec<Vec<usize>>,
}

impl NfaState {
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

    /// Gets a list of transitions, as sets of state indices for each element of the alphabet, in the alphabet's
    /// ordering
    pub fn transitions(&self) -> &[Vec<usize>] {
        self.transitions.as_slice()
    }

    /// Gets the epsilon transitions as a set of state indices
    pub fn epsilon_transitions(&self) -> &[usize] {
        self.epsilon_transitions.as_slice()
    }
}

impl Nfa {
    /// Constructs the intersection of two NFAs, that is, a new NFA that accepts exactly those strings that are accepted
    /// by either the first, the second NFA, or both. This returns `None` if and only if the alphabets of the two NFAs
    /// are unequal (not considering ordering). This is done by adding a new initial state that has epsilon transitions
    /// to both NFAs initial states, and is thus very cheap. In contrast to [Dfa::union] and [Nfa::intersection], this
    /// function intentionally takes ownership over the NFAs since the construction itself is cheap, making cloning a
    /// significant overhead. This function returns an Error with the two provided automatas if and only if the
    /// alphabets of the two automata differs (not considering ordering).
    ///
    /// ```
    /// use dandy::parser;
    /// use dandy::nfa::Nfa;
    ///
    /// let ends_with_aab = "
    ///             a       b
    ///     ->  s1 {s1 s2} {s1}
    ///         s2 {s3}    {}
    ///         s3 {}      {s4}
    ///       * s4 {}      {}
    /// ";
    /// let contains_babb = "
    ///             a    b
    ///     ->  s1 {s1} {s1 s2}
    ///         s2 {s3} {}
    ///         s3 {}   {s4}
    ///         s4 {}   {s5}
    ///       * s5 {s5} {s5}
    /// ";
    /// let ends_with_aab: Nfa = parser::nfa(ends_with_aab).unwrap().try_into().unwrap();
    /// let contains_babb: Nfa = parser::nfa(contains_babb).unwrap().try_into().unwrap();
    ///
    /// // 'any' accepts strings that contains "babb" or ends with "aab".
    /// let any = ends_with_aab.union(contains_babb).unwrap();
    /// assert!(!any.accepts_graphemes("abbabab"));
    /// assert!(any.accepts_graphemes("aaab"));
    /// assert!(any.accepts_graphemes("bbabbaab"));
    /// assert!(any.accepts_graphemes("bbaabaab"));
    /// ```
    pub fn union(mut self, mut other: Self) -> Result<Self, (Self, Self)> {
        if !alphabet_equal(&self.alphabet, &other.alphabet) {
            return Err((self, other));
        }

        let alphabet_translation = other
            .alphabet
            .iter()
            .map(|elem1| {
                self.alphabet
                    .iter()
                    .enumerate()
                    .find_map(|(idx, elem2)| (elem1 == elem2).then_some(idx))
                    .unwrap()
            })
            .collect::<Vec<usize>>();
        // alphabet_translation[i] contains the index for the i'th element of 'other's alphabet in the 'self's alphabet
        // we zip this with the other transition vecs and sort by that

        if !alphabet_translation.windows(2).all(|v| v[0] < v[1]) {
            // We need to re-order the entries
            for state in other.states.iter_mut() {
                state.transitions = {
                    let mut vec = state
                        .transitions
                        .drain(..)
                        .zip(alphabet_translation.iter())
                        .collect::<Vec<_>>();
                    vec.sort_by_key(|(_, b)| **b);
                    vec.into_iter().map(|(a, _)| a).collect()
                };
            }
        }

        let a_states = self.states.len();
        let remapping = |b_idx| Some(b_idx + a_states);
        other.remap_transitions(remapping);

        let b_init = remapping(other.initial_state).unwrap();
        self.states.extend(other.states);

        // Check uniqueness of names
        let names = self
            .states
            .iter()
            .map(|s| s.name.as_ref())
            .collect::<HashSet<_>>();
        if names.len() != self.states.len() {
            // Rename states
            let mut iter = 1..;
            self.states.iter_mut().for_each(|state| {
                state.name = iter
                    .next()
                    .map(|i| Rc::from(i.to_string().as_str()))
                    .unwrap()
            });
        }

        let new_initial_state = NfaState {
            name: self.fresh_name("s_new"),
            initial: true,
            accepting: false,
            epsilon_transitions: vec![self.initial_state, b_init],
            transitions: vec![vec![]; self.alphabet.len()],
        };

        self.states[self.initial_state].initial = false;
        self.states[b_init].initial = false;
        self.initial_state = self.states.len();
        self.states.push(new_initial_state);
        Ok(self)
    }

    /// Constructs the intersection of two NFAs, that is, a new NFA that accepts exactly those strings that are accepted
    /// by both the first and second NFAs. This returns `None` if and only if the alphabets of the two NFAs are unequal
    /// (not considering ordering). This is done by the product construction.
    ///
    /// ```
    /// use dandy::parser;
    /// use dandy::nfa::Nfa;
    ///
    /// let ends_with_aab = "
    ///             a       b
    ///     ->  s1 {s1 s2} {s1}
    ///         s2 {s3}    {}
    ///         s3 {}      {s4}
    ///       * s4 {}      {}
    /// ";
    /// let contains_babb = "
    ///             a    b
    ///     ->  s1 {s1} {s1 s2}
    ///         s2 {s3} {}
    ///         s3 {}   {s4}
    ///         s4 {}   {s5}
    ///       * s5 {s5} {s5}
    /// ";
    /// let ends_with_aab: Nfa = parser::nfa(ends_with_aab).unwrap().try_into().unwrap();
    /// let contains_babb: Nfa = parser::nfa(contains_babb).unwrap().try_into().unwrap();
    ///
    /// // 'both' accepts strings that contains "babb" and ends with "aab".
    /// let both = ends_with_aab.intersection(&contains_babb).unwrap();
    /// assert!(!both.accepts_graphemes("aaab"));
    /// assert!(!both.accepts_graphemes("abbabb"));
    /// assert!(both.accepts_graphemes("bbabbaab"));
    /// ```
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        self.product_construction(other, |s1, s2| {
            s1.zip(s2)
                .map_or(false, |(s1, s2)| s1.accepting && s2.accepting)
        })
    }

    /// Constructs a new NFA from two NFAs using the product construction. That is a new NFA with states corresponding
    /// to both the state the first NFA and the second NFA would be in on any given input. If that state is an accepting
    /// state or not is given by the `combinator` function, combining the state from the first parser and the second
    /// parser. Since it is possible for a NFA to be in "no states" upon a certain point in evaluation over a string,,
    /// the combinator takes two optional states. The pair `(None, Some(state: A))` corresponds to the point where
    /// NFA 1 is in no states, and NFA 2 is in state A.
    ///
    /// Note that the usages for this construction isn't the same as the usages for the product construction for DFAs:
    /// one cannot achieve negations on a NFA by complementing the set of final states, and in the same way, one cannot
    /// use the product construction to get the difference or symmetric difference. Using the combinator
    /// `|s1, s2| s1.map_or(false, s1.accepting) && !s2.map_or(false, s2.accepting)` doesn't produce a NFA accepting
    /// words which are accepted by the first NFA but not by the second (as it would on DFAs), rather the constructed
    /// NFA will accept any words which leads to the first NFA accepting it and the second NFA being in at least one
    /// state which is not accepting (which is not equivalent to the second NFA not accepting the word). Product
    /// constructions can only be used to calculate the union or intersection of two NFAs, since that doesn't include
    /// negation of any of the NFAs. However, for unions, there is a more efficient construction, see [Nfa::union].
    ///
    /// If the alphabets of the provided automata differs, this function returns `None`.
    pub fn product_construction(
        &self,
        other: &Self,
        mut combinator: impl FnMut(Option<&NfaState>, Option<&NfaState>) -> bool,
    ) -> Option<Self> {
        // If alphabets differ, we can't make a product construction
        if !alphabet_equal(&self.alphabet, &other.alphabet) {
            return None;
        }

        let alphabet_translation = self
            .alphabet
            .iter()
            .map(|elem1| {
                other
                    .alphabet
                    .iter()
                    .enumerate()
                    .find_map(|(idx, elem2)| (elem1 == elem2).then_some(idx))
                    .unwrap()
            })
            .collect::<Vec<usize>>();

        // initially, we explore the (pair of) initial states
        let q1 = self.initial_state;
        let q2 = other.initial_state;
        let mut state_pairs_to_explore = vec![(Some(q1), Some(q2))];
        let mut explored_states = HashSet::new();
        explored_states.insert((Some(q1), Some(q2)));

        // maps (q1, q2) to accepting?
        // state_data elements is (state_pair, accepting, transitions, epsilon_transitions)
        let mut state_data = vec![];

        while let Some((s1, s2)) = state_pairs_to_explore.pop() {
            let mut transition_list = Vec::with_capacity(self.alphabet.len());
            let mut eps_transitions = Vec::with_capacity(
                s1.map_or(0, |s1| self.states[s1].epsilon_transitions.len())
                    + s2.map_or(0, |s2| other.states[s2].epsilon_transitions.len()),
            );

            for elem in 0..self.alphabet.len() {
                let other_elem = alphabet_translation[elem];

                let mut elem_transitions = Vec::with_capacity(
                    s1.map_or(1, |s1| self.states[s1].transitions[elem].len())
                        * s2.map_or(1, |s2| other.states[s2].transitions[other_elem].len()),
                );

                match (
                    s1.filter(|&idx| !self.states[idx].transitions[elem].is_empty()),
                    s2.filter(|&idx| !other.states[idx].transitions[other_elem].is_empty()),
                ) {
                    (Some(s1), Some(s2)) => {
                        let on_elem1 = &self.states[s1].transitions[elem];
                        let on_elem2 = &other.states[s2].transitions[other_elem];

                        for &tr1 in on_elem1 {
                            for &tr2 in on_elem2 {
                                let states = (Some(tr1), Some(tr2));
                                elem_transitions.push(states);
                                if explored_states.insert(states) {
                                    state_pairs_to_explore.push(states);
                                }
                            }
                        }
                    }

                    (Some(s1), None) => {
                        let on_elem1 = &self.states[s1].transitions[elem];
                        for &tr1 in on_elem1 {
                            let states = (Some(tr1), None);
                            elem_transitions.push(states);
                            if explored_states.insert(states) {
                                state_pairs_to_explore.push(states);
                            }
                        }
                    }

                    (None, Some(s2)) => {
                        let on_elem2 = &other.states[s2].transitions[other_elem];
                        for &tr2 in on_elem2 {
                            let states = (None, Some(tr2));
                            elem_transitions.push(states);
                            if explored_states.insert(states) {
                                state_pairs_to_explore.push(states);
                            }
                        }
                    }

                    (None, None) => {}
                }

                transition_list.push(elem_transitions);
            }

            if let Some(s1) = s1 {
                for &eps1 in &self.states[s1].epsilon_transitions {
                    let states = (Some(eps1), s2);
                    eps_transitions.push(states);
                    if explored_states.insert(states) {
                        state_pairs_to_explore.push(states);
                    }
                }
            }

            if let Some(s2) = s2 {
                for &eps2 in &other.states[s2].epsilon_transitions {
                    let states = (s1, Some(eps2));
                    eps_transitions.push(states);
                    if explored_states.insert(states) {
                        state_pairs_to_explore.push(states);
                    }
                }
            }

            state_data.push((
                (s1, s2),
                combinator(
                    s1.map(|s1| &self.states[s1]),
                    s2.map(|s2| &other.states[s2]),
                ),
                transition_list,
                eps_transitions,
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
                        s1.map_or("none", |s1| &self.states[s1].name),
                        s2.map_or("none", |s2| &other.states[s2].name)
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
            .map(|(idx, ((s1, s2), _, _, _))| ((*s1, *s2), idx))
            .collect::<HashMap<_, _>>();
        let initial_state = *rev_state_idx_map
            .get(&(Some(q1), Some(q2)))
            .expect("Initial state should have an index");

        let states = state_data
            .into_iter()
            .map(
                |(states, accepting, transitions, epsilon_transitions)| NfaState {
                    name: names
                        .get(&states)
                        .expect("All states should have a name")
                        .clone(),
                    initial: states == (Some(q1), Some(q2)),
                    accepting,
                    transitions: transitions
                        .into_iter()
                        .map(|transition_list| {
                            transition_list
                                .iter()
                                .map(|states| {
                                    *rev_state_idx_map.get(&states).expect(
                                        "Each state pair with transition to it should have a idx",
                                    )
                                })
                                .collect()
                        })
                        .collect(),
                    epsilon_transitions: epsilon_transitions
                        .into_iter()
                        .map(|states| {
                            *rev_state_idx_map
                                .get(&states)
                                .expect("Each state pair with transition to it should have a idx")
                        })
                        .collect(),
                },
            )
            .collect::<Vec<_>>();
        Some(Nfa {
            alphabet: self.alphabet.clone(),
            states,
            initial_state,
        })
    }

    /// Optimizes this NFA by first removing all unreachable states and then removing all epsilon moves. This simply
    /// executes [Nfa::remove_unreachable_states] and then [Nfa::remove_epsilon_moves]. See documentation of those
    /// functions for more information.
    pub fn optimize(&mut self) {
        self.remove_unreachable_states();
        self.remove_epsilon_moves();
    }

    /// Removes all epsilon moves from this NFA, and after this call returns, no state will have any epsilon moves and
    /// [Nfa::has_epsilon_moves] will return false. This is done by adding the epsilon closure of each state to each
    /// transition to that state, then removing all epsilon transitions from all states. Additionally, this function
    /// removes all "dead states", which is defined to be states which has no non-epsilon transitions or where all
    /// non-epsilon transitions are to other dead states (after expanding the epsilon closures). The initial state
    /// and accepting states are never considered dead.
    ///
    /// If the initial state has epsilon moves to non-dead states, the NFA effectively has "multiple initial states"
    /// which isn't allowed per definition of NFA. An additional state is then added, which is then promoted to start
    /// state, having the same behaviour as the previous initial state. This demotion from being an initial state to
    /// a non-initial state may have demoted it to being considered a "dead state", in which case it is removed.
    ///
    /// This procedure makes sure no states without any normal transitions exists in the automata after execution. Note
    /// that this procedure isn't a minimization of the NFA, nor does it remove unreachable states. See
    /// [Nfa::remove_unreachable_states] for removing unreachable states.
    ///
    /// ```
    /// use dandy::nfa::Nfa;
    /// use dandy::parser;
    ///
    /// let contains_eps_moves = "
    ///        eps  a    b    c
    /// -> s0 {}   {s1} {s1} {s2} # This is not a dead state, it is initial and has transitions to s2
    ///    s1 {s0} {}   {}   {}   # This is a dead state, it only has epsilon transitions
    ///  * s2 {s1} {s2} {s2} {s2} # This is not a dead state, it is accepting
    ///    s3 {s3} {s1} {s1} {s1} # Not dead, since closure of {s1} is {s0 s1}, and {s0} isn't dead
    /// ";
    /// let mut nfa: Nfa = parser::nfa(contains_eps_moves).unwrap().try_into().unwrap();
    /// nfa.remove_epsilon_moves();
    /// assert_eq!(nfa.states().len(), 3);
    /// assert_eq!(nfa.states()[0].name(), "s0");
    /// assert_eq!(nfa.states()[1].name(), "s2");
    /// assert_eq!(nfa.states()[2].name(), "s3");
    ///
    /// // Here is an example requiring some more explanation:
    /// // i is not a dead state since it is the initial state
    /// // Clearly, i0 is a dead state
    /// // i1 may look like a dead state, but it has transitions to i which is initial
    /// // i2 only has transitions to the dead state i0, so it is considered dead
    /// // i3 may seem dead but after inlining the eps-closure of i1, it actually
    /// //   has transitions to y on a, b and c, so it is not considered dead.
    /// //   If we would have removed this state, it might have changed the language
    /// //   of the NFA (in this case, it isn't reachable anyways).
    /// // A new initial state is created (since the epsilon closure of the current
    /// //   initial state has more than one element), and 'i' is demoted to non-initial.
    /// // All of a sudden, i is considered dead as well since it only has non-eps
    /// //   transitions to dead states. i1, however, is not dead since inlining the
    /// //   eps-closure if 'i' gives a transition to 'y' upon seeing 'c'
    /// // Since 'i' is now considered dead, the new initial state will take its name.
    /// // The non-dead states are 'i1', 'i3', 'x', 'y' and the new initial state 'i'.
    /// let branching_initial_state = "
    ///        eps        a       b       c
    /// -> i  {i0 i1 i2} {i0}    {i2}    {i0 i2}
    ///    i0 {i0}       {}      {}      {}
    ///    i1 {y}        {i0}    {i0}    {i}
    ///    i2 {}         {}      {i0}    {i0}
    ///    i3 {i1}       {i1}    {i1}    {i1}
    ///    x  {x}        {y}     {x}     {x}
    ///  * y  {}         {x}     {y}     {y}
    /// ";
    /// let mut nfa: Nfa = parser::nfa(branching_initial_state).unwrap().try_into().unwrap();
    /// nfa.remove_epsilon_moves();
    /// println!("{}", nfa.ascii_table());
    /// assert_eq!(nfa.states().len(), 5);
    /// assert_eq!(nfa.states()[0].name(), "i1");
    /// assert_eq!(nfa.states()[1].name(), "i3");
    /// assert_eq!(nfa.states()[2].name(), "x");
    /// assert_eq!(nfa.states()[3].name(), "y");
    /// assert_eq!(nfa.states()[4].name(), "i"); // The new initial state is placed last
    /// ```
    pub fn remove_epsilon_moves(&mut self) {
        if !self.has_epsilon_moves() {
            return;
        }

        // Pre-calculate all epsilon closures
        let closures = (0..self.states.len())
            .filter_map(|idx| self.closure(idx))
            .collect::<Vec<_>>();

        // first, inline all epsilon closures
        self.states.iter_mut().for_each(|state| {
            state.transitions.iter_mut().for_each(|transition_set| {
                // On transition from a to b, transition from a to eps closure of b
                *transition_set = transition_set
                    .iter()
                    .fold(HashSet::new(), |mut set, transition| {
                        set.extend(&closures[*transition]);
                        set
                    })
                    .drain()
                    .collect();
            });
            state.epsilon_transitions.clear();
        });

        // Secondly, find all 'dead states', i.e. states without normal
        // transitions that we can remove later on.
        let mut dead_states = HashSet::new();

        let mut added_states = true;
        while added_states {
            added_states = false;
            for (idx, state) in self.states.iter().enumerate() {
                if !dead_states.contains(&idx)
                    && !state.is_accepting()
                    && !state.is_initial()
                    && state
                        .transitions
                        .iter()
                        .all(|transitions| transitions.iter().all(|idx| dead_states.contains(idx)))
                {
                    dead_states.insert(idx);
                    added_states = true;
                }
            }
        }

        // Thirdly, figure out if we need a new initial state (which we would need if
        // our initial state has epsilon transitions to other than dead states)
        let init_closure = closures[self.initial_state]
            .iter()
            .copied()
            .filter(|x| !dead_states.contains(x))
            .collect::<HashSet<_>>();
        if init_closure.len() > 1 {
            // We see that the epsilon closure of the initial state includes more than 1 state, so make sure to remove
            // it as well!

            // Check if the old initial state is now dead, in that case we can re-use its name
            let old_initial = self.initial_state;
            self.states[old_initial].initial = false;
            let old_state_dead = !self.states[old_initial].accepting
                && self.states[old_initial]
                    .transitions
                    .iter()
                    .all(|transitions| transitions.iter().all(|idx| dead_states.contains(idx)));

            if old_state_dead {
                // Remove it, and re-run dead state search since this may reveal more dead states!
                dead_states.insert(old_initial);
                let mut added_states = true;
                while added_states {
                    added_states = false;
                    for (idx, state) in self.states.iter().enumerate() {
                        if !dead_states.contains(&idx)
                            && !state.is_accepting()
                            && !state.is_initial()
                            && state.transitions.iter().all(|transitions| {
                                transitions.iter().all(|idx| dead_states.contains(idx))
                            })
                        {
                            dead_states.insert(idx);
                            added_states = true;
                        }
                    }
                }
            }

            let new_state_name = if old_state_dead {
                self.states[old_initial].name.clone()
            } else {
                self.fresh_name("s_new")
            };

            let transitions = (0..self.alphabet.len())
                .map(|elem_idx| {
                    init_closure
                        .iter()
                        .fold(HashSet::new(), |mut set, &state| {
                            set.extend(self.states[state].transitions[elem_idx].iter().copied());
                            set
                        })
                        .drain()
                        .filter(|i| !dead_states.contains(i))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            let new_state = NfaState {
                name: new_state_name,
                initial: true,
                accepting: init_closure.iter().any(|idx| self.states[*idx].accepting),
                epsilon_transitions: vec![],
                transitions,
            };
            self.states[old_initial].initial = false;
            self.initial_state = self.states.len();
            self.states.push(new_state);
        }

        // Then, remove all dead states from transition tables
        self.states.iter_mut().for_each(|state| {
            state
                .transitions
                .iter_mut()
                .for_each(|transition| transition.retain(|idx| !dead_states.contains(idx)))
        });

        // Finally, remove all dead states
        self.remove_states(dead_states.drain().collect());
    }

    /// This function removes the states with indices in the vector from this NFA, changing the transition tables
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

    /// Remaps the transitions so that any transition and epsilon transition to n gets mapped to mapper(n)
    /// (if any, otherwise n is preserved)
    fn remap_transitions(&mut self, mapper: impl Fn(usize) -> Option<usize>) {
        self.states.iter_mut().for_each(|state| {
            state.transitions.iter_mut().for_each(|table| {
                table
                    .iter_mut()
                    .for_each(|trans| *trans = mapper(*trans).unwrap_or(*trans))
            });
            state
                .epsilon_transitions
                .iter_mut()
                .for_each(|trans| *trans = mapper(*trans).unwrap_or(*trans));
        })
    }

    fn fresh_name(&mut self, wanted: &str) -> Rc<str> {
        if self.states.iter().all(|s| s.name.as_ref() != wanted) {
            Rc::from(wanted)
        } else {
            (0..)
                .map(|i| Rc::from(i.to_string().as_str()))
                .find(|n| self.states.iter().all(|s| &s.name != n))
                .unwrap()
        }
    }

    /// Removes the unreachable states of this NFA, that is, all states that cannot be reached by any input to
    /// the automata. See [Nfa::unreachable_states] to get the unreachable states
    pub fn remove_unreachable_states(&mut self) {
        let states = self.unreachable_state_idx().into_iter().collect();
        self.remove_states(states);
    }

    /// Finds the unreachable states, that is, all states that cannot be reached by any input to the automata
    pub fn unreachable_states(&self) -> Vec<&NfaState> {
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

    /// Checks if this NFA has an accepting state that is reachable from the initial state, that is, if it has some
    /// input which it accepts
    pub fn has_reachable_accepting_state(&self) -> bool {
        // Use _idx to not allocate Vec
        self.reachable_state_idx()
            .iter()
            .any(|idx| self.states[*idx].accepting)
    }

    /// Finds the reachable states, that is, all states that can be reached by some input to the automata
    pub fn reachable_states(&self) -> Vec<&NfaState> {
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
                .flat_map(|state| {
                    // For each state, add all transitions and its epsilon closure
                    self.states[state]
                        .transitions
                        .iter()
                        .flatten()
                        .copied()
                        .chain(self.closure(state).unwrap().into_iter())
                })
                .filter(|&state| reachables.insert(state))
                .collect();
        }
        reachables
    }

    /// Iterate over the words accepted by this NFA in lexicographic order (according to
    /// the order of the alphabet). The words are represented by a `Vec` of indices of the
    /// elements, corresponding to the same element in the alphabet. For a `Vec` of `Rc<str>`s,
    /// see [Nfa::word_components], and for a `Vec` of element indices, see [Nfa::word_component_indices].
    /// Notably, this operation does not include a NFA-to-DFA conversion and doesn't suffer
    /// from exponential blowups.
    ///
    /// *NOTE:* Current implementation only works for NFAs without epsilon moves.
    /// See [Nfa::remove_epsilon_moves]
    pub fn words(&self) -> Words {
        Words::new(self)
    }

    /// Iterate over the words accepted by this NFA in lexicographic order (according to
    /// the order of the alphabet). The words are represented by a `Vec` of indices of the
    /// elements, corresponding to the same element in the alphabet. For a String
    /// representation, see [Nfa::words], and for a `Vec` of element indices, see [Nfa::word_component_indices].
    /// Notably, this operation does not include a NFA-to-DFA conversion and doesn't suffer
    /// from exponential blowups.
    ///
    /// *NOTE:* Current implementation only works for NFAs without epsilon moves.
    /// See [Nfa::remove_epsilon_moves]
    pub fn word_components(&self) -> WordComponents {
        WordComponents::new(self)
    }

    /// Iterate over the words accepted by this NFA in lexicographic order (according to
    /// the order of the alphabet). The words are represented by a `Vec` of indices of the
    /// elements, corresponding to the same element in the alphabet. For a String
    /// representation, see [words], and for a `Vec` of `Rc<str>`, see [Nfa::word_components].
    /// Notably, this operation does not include a NFA-to-DFA conversion and doesn't suffer
    /// from exponential blowups.
    ///
    /// *NOTE:* Current implementation only works for NFAs without epsilon moves.
    /// See [Nfa::remove_epsilon_moves]
    pub fn word_component_indices(&self) -> WordComponentIndices {
        WordComponentIndices::new(self)
    }

    /// Converts this NFA to a DFA using the subset construction.
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
                name: Rc::from(n.to_string()),
                initial: n == 0,
                accepting: accepting.contains(&n),
                transitions: transitions.remove(key).unwrap(),
            })
            .collect();

        Dfa {
            alphabet: self.alphabet.clone(), // Clone is cheap: alphabet is Rc<_>
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

    /// Checks if this automaton accepts the given string of graphemes, if every grapheme by
    /// itself is considered as an element of the alphabet. Note that if the alphabet contains
    /// elements with multiple graphemes, those won't be recognized. To check if there are
    /// elements with multiple graphemes, see [Nfa::graphemes_only]. A grapheme is defined to be
    /// one extended unicode grapheme cluster (which may consist of one or many code points).
    pub fn accepts_graphemes(&self, string: &str) -> bool {
        let graphemes = string.graphemes(true).collect::<Vec<_>>();
        let mut eval = self.evaluator();
        eval.step_multiple(&graphemes);
        eval.is_accepting()
    }

    /// Checks if the alphabet of this automaton consists of only single graphemes. If it does, one may use
    /// [Nfa::accepts_graphemes] instead of [Nfa::accepts] for improved ergonomics. A grapheme is defined to be
    /// one extended unicode grapheme cluster (which may consist of one or many code points).
    pub fn graphemes_only(&self) -> bool {
        self.alphabet
            .iter()
            .all(|str| str.graphemes(true).count() == 1)
    }

    /// Checks if this automaton has any epsilon moves
    pub fn has_epsilon_moves(&self) -> bool {
        self.states
            .iter()
            .any(|state| !state.epsilon_transitions.is_empty())
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
        if !alphabet_equal(&self.alphabet, &other.alphabet) {
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

    /// Gets the alphabet of this NFA
    pub fn alphabet(&self) -> &[Rc<str>] {
        &self.alphabet
    }

    /// Gets the states of this NFA
    pub fn states(&self) -> &[NfaState] {
        self.states.as_slice()
    }

    /// Gets the initial state of this NFA
    pub fn initial_state(&self) -> &NfaState {
        &self.states[self.initial_state]
    }

    /// Get the index of the initial state of this NFA
    pub fn initial_state_index(&self) -> usize {
        self.initial_state
    }
}
