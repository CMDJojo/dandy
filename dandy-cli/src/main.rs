use clap::{Parser, ValueEnum};
use dandy::dfa::Dfa;
use dandy::nfa::Nfa;
use dandy::parser;
use dandy::parser::ParsedDfa;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

/// This command can be used to run operations on DFAs or NFAs.
///
/// `--type dfa/nfa` specifies if you are testing DFAs or NFAs. Defaults to DFA.
/// `--condition all/any` specifies if all lines or just any lines of the files needs to be accepted by the DFA
///     when testing it.
/// The `equivalence` operation checks which of many automata are equivalent to the automata provided.
/// The `test` operation checks which files has all/any lines accepted by the automata.
// Example usage: dandy equivalence tests/dfa1.dfa tests/example_tree/**/*.dfa
#[derive(Parser, Debug)]
#[command(
    version,
    author = "Jonathan Widén",
    about = "A cli tool for importing and checking DFAs and NFAs",
    long_about
)]
struct Args {
    #[arg(
        short,
        long,
        value_enum,
        default_value_t,
        help = "Choose if testing DFAs or NFAs"
    )]
    r#type: FaType,
    #[arg(
        short,
        long,
        value_enum,
        default_value_t,
        help = "(For 'test' operation): Choose if all lines or one line per file must be accepted by the automata"
    )]
    condition: TestType,
    #[arg(
        help = "Choose if you want to check what files define automata equivalent to the given automata, or what files has lines accepted by the automata"
    )]
    operation: OpType,
    #[arg(
        short,
        long,
        default_value_t,
        help = "(For 'equivalence' operation): Requires the DFAs/NFAs to be minimized"
    )]
    minimized: bool,
    #[arg(
        short,
        long,
        default_value_t,
        help = "Output 'true'/'false' rather than a result in text format"
    )]
    r#bool: bool,
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
        Ok(f) => match args.operation {
            OpType::Equivalence => {
                equivalence(&args, &f);
            }
            OpType::Test => {
                test(&args, &f);
            }
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EquivalenceResult {
    FailedToRead(String),
    FailedToParse(String),
    FailedToValidate(String),
    NotEquivalent,
    NotMinimized,
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
            NotMinimized => write!(f, "Equivalent but not minimized"),
            Equivalent => write!(f, "Equivalent"),
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
                let mut dfa: Dfa = dfa.unwrap();
                if args.minimized {
                    let prev_states = dfa.states().len();
                    dfa.minimize();
                    if dfa.states().len() != prev_states {
                        println!("Note: Your input DFA was not minimized. It has been minimized prior to comparing");
                    }
                }
                dfa
            };
            let verify_fn = move |file: &str| -> EquivalenceResult {
                let parse: ParsedDfa = match parser::dfa(file) {
                    Err(e) => return FailedToParse(e.to_string()),
                    Ok(f) => f,
                };
                let other_dfa: Dfa = match Dfa::try_from(parse) {
                    Err(e) => return FailedToValidate(e.to_string()),
                    Ok(f) => f,
                };
                if dfa.equivalent_to(&other_dfa) {
                    if args.minimized && other_dfa.states().len() != dfa.states().len() {
                        // Only if we ask for minimized DFAs and it is not, return NotMinimized
                        NotMinimized
                    } else {
                        Equivalent
                    }
                } else {
                    NotEquivalent
                }
            };
            do_equivalence_check(args, verify_fn);
        }
        FaType::Nfa => {
            if args.minimized {
                eprintln!(
                    "Warn: --minimized can't be used with NFAs, only with DFAs, and is now ignored"
                );
            }
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
                    Ok(f) => f,
                };
                let other_nfa = match Nfa::try_from(parse) {
                    Err(e) => return FailedToValidate(e.to_string()),
                    Ok(f) => f,
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
    let results = args
        .files
        .iter()
        .map(|file| match fs::read_to_string(file) {
            Err(e) => EquivalenceResult::FailedToRead(e.to_string()),
            Ok(f) => verify_fn(&f),
        })
        .zip(args.files.iter())
        .collect::<Vec<_>>();

    results.iter().for_each(|(res, path)| {
        let out = if args.bool {
            (res == &EquivalenceResult::Equivalent).to_string()
        } else {
            res.to_string()
        };
        println!("{}: {}", path.to_string_lossy(), out)
    })
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
    let dfa2 = dandy::parser::dfa(equivalent_dfa)
        .unwrap()
        .try_into()
        .unwrap();
    assert!(dfa.equivalent_to(&dfa2));
}
