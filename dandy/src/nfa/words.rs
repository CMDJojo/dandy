use crate::nfa::Nfa;
use nalgebra::DMatrix;
use num_traits::{One, Zero};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Mul, MulAssign};
use std::rc::Rc;
use NumBool::*;

/// An iterator visiting all words accepted by a NFA iteratively, returning them as [String]s. The
/// iterator visits words in lexicographic order, according to the alphabet of the NFA.
pub struct Words<'a> {
    inner: WordComponentIndices<'a>,
}

impl Iterator for Words<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.put_next();
        self.inner.last_word.as_ref().map(|components| {
            components
                .iter()
                .fold(String::with_capacity(components.len()), |mut s, c| {
                    s.push_str(&self.inner.nfa.alphabet[*c]);
                    s
                })
        })
    }
}

impl<'a> Words<'a> {
    pub fn new(nfa: &'a Nfa) -> Self {
        Self {
            inner: WordComponentIndices::new(nfa),
        }
    }
}

/// An iterator visiting all words accepted by a NFA iteratively, returning them as vectors of
/// components (`Rc<str>`) for elements of the words. The iterator visits words in lexicographic
/// order, according to the alphabet of the NFA.
pub struct WordComponents<'a> {
    inner: WordComponentIndices<'a>,
}

impl Iterator for WordComponents<'_> {
    type Item = Vec<Rc<str>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.put_next();
        self.inner.last_word.as_ref().map(|components| {
            components
                .iter()
                .map(|c| self.inner.nfa.alphabet[*c].clone())
                .collect()
        })
    }
}

impl<'a> WordComponents<'a> {
    pub fn new(nfa: &'a Nfa) -> Self {
        Self {
            inner: WordComponentIndices::new(nfa),
        }
    }
}

/// An iterator visiting all words accepted by a NFA iteratively, returning them as vectors of
/// indices for elements of the words. The iterator visits words in lexicographic
/// order, according to the alphabet of the NFA.
pub struct WordComponentIndices<'a> {
    nfa: &'a Nfa,
    adj_matrices: Vec<DMatrix<NumBool>>,
    final_states: HashSet<usize>,
    state_stack: Vec<HashSet<usize>>,
    #[allow(dead_code)] // Unused for now since we don't support NFAs with epsilon moves yet
    has_epsilon_moves: bool,
    has_failed: bool,
    last_word: Option<Vec<usize>>,
}

impl Iterator for WordComponentIndices<'_> {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        self.put_next();
        self.last_word.clone()
    }
}

// Based on: http://maya-ackerman.com/wp-content/uploads/2018/09/Enumeration_AckermanShallit2.pdf
impl<'a> WordComponentIndices<'a> {
    fn put_next(&mut self) {
        if self.has_failed {
            return;
        }

        let mut len = 0;
        let mut num_cec = 0;

        if let Some(last) = self.last_word.clone() {
            len = last.len() + 1;
            if let Some(new) = self.next_word(last) {
                self.last_word = Some(new);
                return;
            }
        }

        while num_cec < self.nfa.states.len() {
            self.state_stack.clear();
            self.state_stack
                .push(HashSet::from([self.nfa.initial_state]));
            match self.min_word(len) {
                None => {
                    num_cec += 1;
                    len += 1;
                }
                Some(w) => {
                    self.last_word = Some(w);
                    return;
                }
            }
        }
        self.last_word = None;
        self.has_failed = true;
    }

    fn next_word(&mut self, mut word: Vec<usize>) -> Option<Vec<usize>> {
        let WordComponentIndices {
            nfa,
            state_stack,
            ..
        } = self;
        let n_complete = |n, from| {
            let mut s: HashSet<usize> = HashSet::new();
            s.insert(from);
            for _ in 0..n {
                s = s
                    .into_iter()
                    .flat_map(|i| nfa.states[i].transitions.iter().flatten())
                    .copied()
                    .collect()
            }
            s.into_iter().any(|idx| nfa.states[idx].is_accepting())
        };

        for i in (1..=word.len()).rev() {
            let current_s = state_stack.last().unwrap();
            let r = current_s
                .iter()
                .flat_map(|i| nfa.states[*i].transitions.iter().flatten())
                .copied()
                .filter(|v| n_complete(word.len() - i, *v))
                .collect::<HashSet<_>>();
            // r is all states that we can get to from one step from S to reach F in (n-i) moves

            let a = (0..nfa.alphabet.len())
                .filter(|idx| {
                    let lhs = current_s
                        .iter()
                        .flat_map(|u| nfa.states[*u].transitions[*idx].iter())
                        .copied()
                        .collect::<HashSet<_>>();
                    lhs.intersection(&r).count() > 0
                })
                .collect::<Vec<_>>();

            if a.iter().all(|a| *a <= word[i - 1]) {
                state_stack.pop();
            } else {
                let b = *a.iter().find(|&a| *a > word[i - 1]).unwrap();

                let s = current_s
                    .iter()
                    .flat_map(|i| nfa.states[*i].transitions[b].iter())
                    .copied()
                    .filter(|v| n_complete(word.len() - i, *v))
                    .collect::<HashSet<_>>();

                let n = word.len();
                word.truncate(i - 1);
                word.push(b);

                return if i != n {
                    state_stack.push(s);
                    word.extend(self.min_word(n - i).unwrap());
                    Some(word)
                } else {
                    Some(word)
                };
            }
        }
        None
    }

    /// Gets the minimum word of size n, and updates the stack S
    fn min_word(&mut self, n: usize) -> Option<Vec<usize>> {
        self.generate_matrices_up_to(n);
        let mut current_s = self
            .state_stack
            .last()
            .expect("min_word: state stack should be nonempty");

        // If there is no way from a current state to the end
        if current_s.iter().copied().all(|from| {
            self.final_states
                .iter()
                .copied()
                .all(|to| self.adj_matrices[n][(from, to)] == False)
        }) {
            return None;
        }

        let mut ret = Vec::with_capacity(n); // this might be underestimating
        for i in 0..n {
            let matrix = &self.adj_matrices[n - i - 1];
            let next_elem_idx = (0..self.nfa.alphabet.len())
                .find(|elem_idx| {
                    current_s.iter().any(|u| {
                        self.final_states.iter().any(|f| {
                            self.nfa.states[*u].transitions[*elem_idx]
                                .iter()
                                .any(|v| matrix[(*v, *f)] == True)
                        })
                    })
                })
                .unwrap();
            ret.push(next_elem_idx);

            if i != n - 1 {
                let mut new_s = current_s.iter().fold(HashSet::new(), |mut set, idx| {
                    set.extend(self.nfa.states[*idx].transitions[next_elem_idx].iter());
                    set
                });
                new_s.retain(|v| self.final_states.iter().any(|f| matrix[(*v, *f)] == True));
                self.state_stack.push(new_s);
                current_s = self.state_stack.last().unwrap();
            }
        }

        Some(ret)
    }

    fn is_reachable_in_one_step(nfa: &'a Nfa, from: usize, to: usize, epsilon_moves: bool) -> bool {
        if epsilon_moves {
            nfa.closure(from)
                .expect("'from' state should exist")
                .into_iter()
                .any(|from_intermediate| {
                    nfa.states[from_intermediate]
                        .transitions()
                        .iter()
                        .any(|on_symbol| {
                            on_symbol
                                .iter()
                                .copied()
                                .any(|destination| nfa.closure(destination).unwrap().contains(&to))
                        })
                })
        } else {
            nfa.states[from]
                .transitions
                .iter()
                .any(|on_symbol| on_symbol.contains(&to))
        }
    }

    fn generate_matrices_up_to(&mut self, n: usize) {
        while self.adj_matrices.len() <= n {
            self.adj_matrices
                .push(&self.adj_matrices[1] * self.adj_matrices.last().unwrap())
        }
    }

    /// Generates the adjacency matrix M, where M(i, j) = 1 iff there is a transition from state
    /// with index i to state with index j using exactly one character from the alphabet. This means
    /// that state a is adjacent to state b if we can move from any state in the epsilon closure of
    /// a to any state whose epsilon closure includes b upon seeing one symbol from the alphabet.
    fn generate_adjacency_matrix(nfa: &'a Nfa) -> DMatrix<NumBool> {
        let n = nfa.states.len();
        let eps = nfa.has_epsilon_moves();
        DMatrix::from_fn(n, n, |from, to| {
            Self::is_reachable_in_one_step(nfa, from, to, eps).into()
        })
    }

    fn identity_matrix(n: usize) -> DMatrix<NumBool> {
        DMatrix::from_fn(n, n, |x, y| (x == y).into())
    }

    pub fn new(nfa: &'a Nfa) -> Self {
        let final_states = nfa
            .states
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.accepting.then_some(i))
            .collect();
        let has_epsilon_moves = nfa.has_epsilon_moves();
        if has_epsilon_moves {
            unimplemented!("Words iterator for NFAs with epsilon moves is unimplemented");
        }
        Self {
            nfa,
            adj_matrices: vec![
                Self::identity_matrix(nfa.states.len()),
                Self::generate_adjacency_matrix(nfa),
            ],
            final_states,
            state_stack: vec![],
            has_epsilon_moves,
            has_failed: false,
            last_word: None,
        }
    }
}

impl<'a> From<&'a Nfa> for WordComponentIndices<'a> {
    fn from(nfa: &'a Nfa) -> Self {
        Self::new(nfa)
    }
}

/// A type equal to `bool` in terms of bit pattern and size, but implementing num traits like
/// zero (false) and one (true), add (false + false = false, _ = true), mul (true * true = true,
/// _ = false). This is to be able to have a matrix of bools, since matrix multiplication requires
/// those implementations.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NumBool {
    False = 0,
    True = 1,
}

impl Add for NumBool {
    type Output = NumBool;

    fn add(self, rhs: Self) -> Self::Output {
        if self == False && rhs == False {
            False
        } else {
            True
        }
    }
}

impl AddAssign for NumBool {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Mul for NumBool {
    type Output = NumBool;

    fn mul(self, rhs: Self) -> Self::Output {
        if self == False || rhs == False {
            False
        } else {
            True
        }
    }
}

impl MulAssign for NumBool {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl One for NumBool {
    fn one() -> Self {
        True
    }
}

impl Zero for NumBool {
    fn zero() -> Self {
        False
    }

    fn is_zero(&self) -> bool {
        *self == False
    }
}

impl From<bool> for NumBool {
    fn from(value: bool) -> Self {
        if value {
            True
        } else {
            False
        }
    }
}

impl From<NumBool> for bool {
    fn from(value: NumBool) -> Self {
        value == True
    }
}

impl Display for NumBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if *self == True { '1' } else { '0' })
    }
}
