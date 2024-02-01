# Dandy

[![Crates.io](https://img.shields.io/crates/v/dandy.svg)](https://crates.io/crates/dandy)
[![Docs.rs](https://docs.rs/dandy/badge.svg)](https://docs.rs/dandy)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Dandy is a Rust library for DFA, NFA and ε-NFA automata, strongly based on a text-based file format for automata.

## Usage

```rust
use dandy::dfa::Dfa;

fn main() {
    let raw_dfa = "
           a  b  c
    → * s₀ s₁ s₀ s₂
        s₁ s₂ s₁ s₁
      * s₂ s₂ s₂ s₂
    ";
    // First pass parses without checking validity of the DFA 
    let parsed_dfa = dandy::parser::dfa(raw_dfa).unwrap();
    // Second step checks the existence of all mentioned states and
    // the existence of an initial state
    let dfa: Dfa = parsed_dfa.try_into().unwrap();
    assert!(dfa.accepts(&["a", "b", "c", "c", "a"]));
    assert!(dfa.accepts(&["c", "b", "a"]));
    assert!(!dfa.accepts(&["a", "b", "b", "c"]));

    let equivalent_dfa = "
        a b c
    → * x z x y
      * y y y y
        z y w z
        w y z w
    ";
    let dfa2 = dandy::parser::dfa(equivalent_dfa).unwrap().try_into().unwrap();
    assert!(dfa.equivalent_to(&dfa2));
}
```

## File format

The file format used is more or less just a transition table. The first row (the header) should include
the whole alphabet, and then the rest of the rows should consist of the states, one row for each state.
The row should start with the state name and then, for each element of the alphabet, the transition from
that state upon seeing that element. Before the state name, either -> or → should be used to denote the
initial state, and * to denote that the state is accepting.

Example of a DFA:

```
       a  b  c
→ * s₀ s₁ s₀ s₂
    s₁ s₂ s₁ s₁
  * s₂ s₂ s₂ s₂
```

This table denotes an DFA accepting strings of the alphabet 'a', 'b', 'c' with either

* only 'b's
* two 'a's
* a 'c' before the first occurrence of 'a'

Whitespace should be used for delimiters between `→`, `*`, the state name and the transition entries. Lines
containing only whitespace will be ignored, and comments may be added using `#`, ignoring the rest of the
row. Leading and trailing whitespace is ignored. The entries in the do not need to be aligned to the other
rows or the alphabet.

To be a correctly denoted DFA, there must be a transition from each state for each alphabet element. All
states referred to must be defined, and there must be exactly one initial state. There may also not be any
duplicate elements of the alphabet.

The format for NFAs and ε-NFAs is very similar. For each state transition, a set of target states is denoted by
`{`, then the states in a whitespace-separated list, and `}`. To define ε-transitions, the ε character should be
added to the alphabet.

Example of an ε-NFA:

```
     ε    a       b
→ s₀ {}   {s₁}    {s₀ s₂}
  s₁ {s₂} {s₄}    {s₃}
  s₂ {}   {s₁ s₄} {s₃}
  s₃ {s₅} {s₄ s₅} {}
  s₄ {s₃} {}      {s₅}
* s₅ {}   {s₅}    {s₅}
```

Again, whitespace should be used for delimiters between `→`, `*`, the state name and the transition entries.
Whitespace should also be used as a delimiter between entries in each set. Empty transitions (no transitions)
must be written as the empty set `{}`. The same rules for comments and leading and trailing whitespace as for
the DFAs apply. `ε` may be written as "eps", and may be absent for denoting a non-ε-NFA.

## Work-in-progress notes

This crate is very much work-in-progress. The alphabet consists of `String`s. This may be changed to characters or
to a generic type, to facilitate a lower memory footprint and more ergonomic acceptance checking. There are also
very few operations implemented at the moment.

## Operations

This library currently supports:

* Parsing and validating DFAs
* Parsing and validating NFAs (with and without epsilon moves)
* Generating a table suitable for re-parsing of DFAs and NFAs
* Converting DFAs to NFAs, and NFAs to DFAs
* Checking whether two DFAs or two NFAs are equivalent
* Checking if a string is accepted by a DFA or NFA
* Step-by-step evaluation of a string
* Identifying and removing unreachable states from a DFA
* Identifying and merging non-distinguishable states from a DFA
* Minimizing a DFA (by executing the two above-mentioned steps)
* Powerset construction, with a custom combinator or with:
  * Union (boolean or)
  * Intersection (boolean and)
  * Difference (a and not b)
  * Symmetric difference (boolean xor)
* Parsing regexes (validation is done in parsing step)
* Converting regexes to NFAs
