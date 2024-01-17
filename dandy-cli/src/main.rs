use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use dandy::dfa::Dfa;
use dandy::nfa::Nfa;
use dandy::parser;
use dandy::parser::ParsedDfa;

/// This command can be used to run operations on DFAs or NFAs.
///
/// `--type dfa/nfa` specifies if you are testing DFAs or NFAs. Defaults to DFA.
/// `--condition all/any` specifies if all lines or just any lines of the files needs to be accepted by the DFA
///     when testing it.
/// The `equivalence` operation checks which of many automata are equivalent to the automata provided.
/// The `test` operation checks which files has all/any lines accepted by the automata.
// Example usage: dandy equivalence tests/dfa1.dfa tests/example_tree/**/*.dfa
#[derive(Parser, Debug)]
#[command(version, author = "Jonathan Widén", about = "A cli tool for importing and checking DFAs and NFAs", long_about)]
struct Args {
    #[arg(short, long, value_enum, default_value_t)]
    r#type: FaType,
    #[arg(short, long, value_enum, default_value_t)]
    condition: TestType,
    #[arg()]
    operation: OpType,
    #[arg()]
    automata: PathBuf,
    #[arg()]
    files: Vec<PathBuf>,
}

#[derive(Default, Clone, Copy, Debug, ValueEnum)]
enum FaType {
    #[default]
    Dfa,
    Nfa,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OpType {
    Equivalence,
    Test,
}

#[derive(Default, Clone, Copy, Debug, ValueEnum)]
enum TestType {
    #[default]
    All,
    Any,
}

fn main() {
    let args = Args::parse();
    let file = fs::read_to_string(&args.automata);
    match file {
        Err(e) => {
            eprintln!("Error reading input file: {e}");
        }
        Ok(f) => {
            match args.operation {
                OpType::Equivalence => {
                    equivalence(&args, &f);
                }
                OpType::Test => {
                    test(&args, &f);
                }
            }
        }
    }
}

enum EquivalenceResult {
    FailedToRead(String),
    FailedToParse(String),
    FailedToValidate(String),
    NotEquivalent,
    Equivalent,
}

impl Display for EquivalenceResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EquivalenceResult::*;
        match self {
            FailedToRead(s) => write!(f, "Failed to read ({s})"),
            FailedToParse(s) => write!(f, "Failed to parse ({s})"),
            FailedToValidate(s) => write!(f, "Failed to validate ({s})"),
            NotEquivalent => write!(f, "Not Equivalent"),
            Equivalent => write!(f, "Equivalent")
        }
    }
}

fn equivalence(args: &Args, file: &str) {
    use EquivalenceResult::*;
    match args.r#type {
        FaType::Dfa => {
            let dfa: Dfa = {
                let parse = parser::dfa(file);
                if let Err(e) = parse {
                    eprintln!("Error parsing input DFA: {e}");
                    return;
                }
                let dfa = parse.unwrap().try_into();
                if let Err(e) = dfa {
                    eprintln!("Error validating input DFA: {e}");
                    return;
                }
                dfa.unwrap()
            };
            let verify_fn = move |file: &str| -> EquivalenceResult {
                let parse: ParsedDfa = match parser::dfa(file) {
                    Err(e) => return FailedToParse(e.to_string()),
                    Ok(f) => f
                };
                let other_dfa: Dfa = match Dfa::try_from(parse) {
                    Err(e) => return FailedToValidate(e.to_string()),
                    Ok(f) => f
                };
                if dfa.equivalent_to(&other_dfa) {
                    Equivalent
                } else {
                    NotEquivalent
                }
            };
            do_equivalence_check(args, verify_fn);
        }
        FaType::Nfa => {
            let nfa: Nfa = {
                let parse = parser::nfa(file);
                if let Err(e) = parse {
                    eprintln!("Error parsing input NFA: {e}");
                    return;
                }
                let nfa = parse.unwrap().try_into();
                if let Err(e) = nfa {
                    eprintln!("Error validating input NFA: {e}");
                    return;
                }
                nfa.unwrap()
            };
            let verify_fn = move |file: &str| -> EquivalenceResult {
                let parse = match parser::nfa(file) {
                    Err(e) => return FailedToParse(e.to_string()),
                    Ok(f) => f
                };
                let other_nfa = match Nfa::try_from(parse) {
                    Err(e) => return FailedToValidate(e.to_string()),
                    Ok(f) => f
                };
                if nfa.equivalent_to(&other_nfa) {
                    Equivalent
                } else {
                    NotEquivalent
                }
            };
            do_equivalence_check(args, verify_fn);
        }
    }
}

fn do_equivalence_check(args: &Args, verify_fn: impl Fn(&str) -> EquivalenceResult) {
    let results = args.files.iter().map(|file|
        match fs::read_to_string(file) {
            Err(e) => EquivalenceResult::FailedToRead(e.to_string()),
            Ok(f) => verify_fn(&f)
        }
    ).zip(args.files.iter()).collect::<Vec<_>>();

    results.iter().for_each(|(res, path)|
        println!("{}: {}", path.to_string_lossy(), res.to_string())
    )
}

fn test(_args: &Args, _file: &str) {
    todo!()
}

// Code from readme to check validity
#[allow(dead_code)]
fn main1() {
    let raw_dfa = "
           a  b  c
    → * s₀ s₁ s₀ s₂
        s₁ s₂ s₁ s₁
      * s₂ s₂ s₂ s₂
    ";
    let parsed_dfa = dandy::parser::dfa(raw_dfa).unwrap();
    let dfa: Dfa = parsed_dfa.try_into().unwrap();
    assert!(dfa.accepts(&vec!["a", "b", "c", "c", "a"]));
    assert!(dfa.accepts(&vec!["c", "b", "a"]));
    assert!(!dfa.accepts(&vec!["a", "b", "b", "c"]));

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

// Temporary code for testing
#[allow(dead_code)]
fn main2() {
    println!("Hello, world!");
    let dfa = dandy::parser::dfa(include_str!("example.dfa"));
    dbg!(&dfa);
    if let Ok(dfa) = dfa {
        let actual_dfa = dandy::dfa::Dfa::try_from(dfa);
        dbg!(&actual_dfa);
        if let Err(e) = actual_dfa {
            println!("{}", e)
        }
    }

    let dfa1: dandy::dfa::Dfa = dandy::parser::dfa(include_str!("example.dfa"))
        .unwrap()
        .try_into()
        .unwrap();

    let dfa2: dandy::dfa::Dfa = dandy::parser::dfa(include_str!("example2.dfa"))
        .unwrap()
        .try_into()
        .unwrap();

    println!("DFA1!!!!");
    dbg!(&dfa1);
    println!("DFA2!!!!");
    dbg!(&dfa2);

    println!("{}", dfa1.equivalent_to(&dfa2));

    println!("{}", dfa2.to_table());

    let nfa = dandy::parser::full_nfa(include_str!("example.nfa")).unwrap().1;
    let nfa: dandy::nfa::Nfa = nfa.try_into().unwrap();
    let output = nfa.to_table();
    println!("{output}");

    let nfa2 = dandy::parser::nfa(include_str!("example2.nfa")).unwrap();
    let nfa2: dandy::nfa::Nfa = nfa2.try_into().unwrap();
    let output = nfa2.to_table();
    println!("{output}");

    let nfa3 = dandy::parser::nfa(include_str!("example3.nfa")).unwrap();
    let nfa3: dandy::nfa::Nfa = nfa3.try_into().unwrap();
    let output = nfa3.to_table();
    println!("{output}");

    println!("{}", dfa1.equivalent_to(&dfa2));
    println!("{}", nfa2.equivalent_to(&nfa3));

    println!("{}", dfa2.to_table());
    println!("{}", dfa2.to_nfa().to_table());

    println!("Converted: (subset construction)");
    println!("{}", nfa2.to_dfa().to_table());

    let eq_nfa_dfa = parser::dfa(include_str!("eq_example2_nfa.dfa"))
        .unwrap();
    let eq_nfa_dfa: Dfa = eq_nfa_dfa.try_into().unwrap();
    println!("Other:");
    println!("{}", eq_nfa_dfa.to_table());

    println!("Equivalent: {}", eq_nfa_dfa.equivalent_to(&nfa2.to_dfa()))
}
