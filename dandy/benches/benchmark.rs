use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dandy::dfa::Dfa;
use dandy::parser;
use lazy_static::lazy_static;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::TestRunner;
use regex::Regex as LibRegex;
use std::fs;
use std::path::Path;

lazy_static! {
    static ref DFAS: Box<[String]> = {
        (0..10)
            .map(|num| {
                let filename = format!("benches/example_dfas/dfa{num}.txt");
                let path = Path::new(&filename);
                fs::read_to_string(path).unwrap()
            })
            .collect()
    };
    static ref REGEXES: Box<[String]> = {
        (0..10)
            .map(|num| {
                let filename = format!("benches/example_regexes/regex{num}.txt");
                let path = Path::new(&filename);
                fs::read_to_string(path).unwrap()
            })
            .collect()
    };
}

pub fn powerset(c: &mut Criterion) {
    let dfa1: Dfa = parser::dfa(&DFAS[0]).unwrap().try_into().unwrap();
    let dfa2: Dfa = parser::dfa(&DFAS[1]).unwrap().try_into().unwrap();
    c.bench_function("union", |b| b.iter(|| dfa1.union(black_box(&dfa2))));
    c.bench_function("intersection", |b| {
        b.iter(|| dfa1.intersection(black_box(&dfa2)))
    });
    c.bench_function("difference", |b| {
        b.iter(|| dfa1.difference(black_box(&dfa2)))
    });
    c.bench_function("symmetric difference", |b| {
        b.iter(|| dfa1.symmetric_difference(black_box(&dfa2)))
    });
}

pub fn equivalence_check(c: &mut Criterion) {
    let dfa1: Dfa = parser::dfa(&DFAS[0]).unwrap().try_into().unwrap();
    let dfa2: Dfa = parser::dfa(&DFAS[1]).unwrap().try_into().unwrap();
    c.bench_function("equivalence check", |b| {
        b.iter(|| dfa1.equivalent_to(black_box(&dfa2)))
    });
}

pub fn regex_compile(c: &mut Criterion) {
    c.bench_function("dandy regex compile", |b| {
        b.iter(|| {
            let input = black_box(&REGEXES[6]);
            let regex = parser::regex(input).unwrap();
            let nfa = regex.to_nfa();
            let dfa = nfa.to_dfa();
            dfa
        })
    });

    c.bench_function("library regex compile", |b| {
        b.iter(|| {
            let input = black_box(&REGEXES[6]);
            LibRegex::new(input).unwrap()
        })
    });
}

pub fn regex_check(c: &mut Criterion) {
    let mut runner = TestRunner::default();
    let string_gen = "[a-z]+".new_tree(&mut runner).unwrap();
    let mut regex = parser::regex(&REGEXES[6]).unwrap().to_nfa().to_dfa();
    regex.minimize();

    c.bench_function("dandy regex check", |b| {
        b.iter(|| regex.accepts_graphemes(black_box(&string_gen.current())))
    });

    let mut runner = TestRunner::default();
    let string_gen = "[a-z]+".new_tree(&mut runner).unwrap();
    let input_regex = format!("^({})$", &REGEXES[6]);
    let regex = LibRegex::new(&input_regex).unwrap();
    c.bench_function("library regex check", |b| {
        b.iter(|| regex.is_match(black_box(&string_gen.current())))
    });
}

criterion_group!(
    benches,
    equivalence_check,
    powerset,
    regex_compile,
    regex_check
);
criterion_main!(benches);
