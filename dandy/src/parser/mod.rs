//! # dandy parser
//! This module contains parser implementations for DFAs, NFAs (with and without epsilon transitions) and regular
//! expressions, according to a custom file format.
//!
//! ## Format for DFAs and NFAs
//! The file format for DFAs and NFAs are more or less a text representation of the transition table.
//! The file describing a DFA and NFA should be an UTF-8-encoded file consisting of:
//! - One line containing the alphabet, with whitespace-separated unique elements (the elements may be multiple
//!     characters long). For denoting an NFA with epsilon moves, `ε` or `eps` may be used.
//! - One line for each state, consisting of these whitespace-separated elements, in order:
//!   - Optionally `->` or `→` for denoting that the state is the initial state (there must be exactly one)
//!   - Optionally `*` for denoting that the state is accepting
//!   - The name of the state
//!   - A list of new states to enter upon seeing the corresponding element of the alphabet, one entry for each
//!     element:
//!     - For DFAs, every element must be non-empty, as it must transition once on each new element processed
//!     - For NFAs, it may transition to a set of zero or more states. The set should be delimited by `{` and `}` and
//!       contain a whitespace-separated list of states. `{}` denotes the empty set
//!
//! Here is an example of a DFA:
//! ```text
//!        a  b  c
//! → * s₀ s₁ s₀ s₂
//!     s₁ s₂ s₁ s₁
//!   * s₂ s₂ s₂ s₂
//! ```
//! This table denotes an DFA accepting strings of the alphabet 'a', 'b', 'c' with either
//!
//! - only 'b's,
//! - two 'a's, or
//! - a 'c' before the first occurrence of 'a'
//!
//! Here is an example of a NFA with epsilon moves:
//! ```text
//!      ε    a       b
//! → s₀ {}   {s₁}    {s₀ s₂}
//!   s₁ {s₂} {s₄}    {s₃}
//!   s₂ {}   {s₁ s₄} {s₃}
//!   s₃ {s₅} {s₄ s₅} {}
//!   s₄ {s₃} {}      {s₅}
//! * s₅ {}   {s₅}    {s₅}
//! ```
//! Any lines containing only whitespace are ignored, and if `#` appears on any line, that character and all subsequent
//! characters on that line will be ignored (as a comment).
//!
//! ## Format for Regular Expressions
//! There are eight reserved characters: `∅`, `ε`, `|`, `*`, `+`, `\`, `(` and `)`. Symbols distinct from them
//! may be written as-is. To denote one of the reserved characters, you may escape it with a backslash `\`. Multiple
//! characters in sequence are sequenced (implicit sequence operator). The alternation operator is `|`, Kleene plus
//! and Kleene star are written as `+` and `*`, the empty language is written as `∅`, and the empty string is written
//! as `ε`. Parenthesis is used for grouping `(`/`)`. This is very similar to regex in programming.
//!
//! - `(ab)+c` is a regular expression accepting strings starting with "ab" repeated 1 or many times, followed by "c"
//! - `c(a|b)*c` accepts all strings starting with a `c`, then any amount of `a`s and `b`s, and then a `c`
//!
//! Leading and trailing whitespace is ignored, but not whitespace within the expression itself.
//!

mod fa;
mod regex;

use crate::regex::Regex;
use nom::{combinator::all_consuming, error::Error, Finish};

#[derive(Debug)]
pub struct ParsedNfa<'a> {
    pub head: Vec<NfaAlphabetEntry<'a>>,
    pub states: Vec<ParsedNfaState<'a>>,
}

#[derive(Debug, Clone)]
pub enum NfaAlphabetEntry<'a> {
    Element(&'a str),
    Eps,
}

#[derive(Debug)]
pub struct ParsedNfaState<'a> {
    pub name: &'a str,
    pub initial: bool,
    pub accepting: bool,
    pub transitions: Vec<Vec<&'a str>>,
}

#[derive(Debug)]
pub struct ParsedDfa<'a> {
    pub head: Vec<&'a str>,
    pub states: Vec<ParsedDfaState<'a>>,
}

#[derive(Debug)]
pub struct ParsedDfaState<'a> {
    pub name: &'a str,
    pub initial: bool,
    pub accepting: bool,
    pub transitions: Vec<&'a str>,
}

/// Parses a DFA according to the format above. The whole string must be parsable, otherwise this function errors.
/// Note that the result is a [ParsedDfa], which is not guaranteed to be a valid [crate::dfa::Dfa]. Use
/// [TryInto::try_into] to convert a [ParsedDfa] to a [crate::dfa::Dfa].
pub fn dfa(input: &str) -> Result<ParsedDfa, Error<&str>> {
    all_consuming(fa::full_dfa)(input)
        .finish()
        .map(|(_, dfa)| dfa)
}

/// Parses a NFA according to the format above. The whole string must be parsable, otherwise this function errors.
/// Note that the result is a [ParsedNfa], which is not guaranteed to be a valid [crate::nfa::Nfa]. Use
/// [TryInto::try_into] to convert a [ParsedNfa] to a [crate::nfa::Nfa].
pub fn nfa(input: &str) -> Result<ParsedNfa, Error<&str>> {
    all_consuming(fa::full_nfa)(input)
        .finish()
        .map(|(_, nfa)| nfa)
}

/// Parses a regular expression according to the format above. The whole string must be parsable, otherwise this
/// function errors. All regexes that are successfully parsed by this function is guaranteed to be valid regexes.
pub fn regex(input: &str) -> Result<Regex, Error<&str>> {
    all_consuming(regex::full_regex)(input)
        .finish()
        .map(|(_, regex)| regex)
}
