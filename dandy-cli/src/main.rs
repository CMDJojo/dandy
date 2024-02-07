mod automata;
mod equivalence;
mod intersection;
mod union;

use clap::{Args, Parser, Subcommand, ValueEnum};
use dandy::dfa::parse::DfaParseError;
use dandy::dfa::Dfa;
use dandy::nfa::parse::NfaParseError;
use dandy::nfa::Nfa;
use dandy::parser;
use equivalence::EquivalenceResult;
use std::fs;
use std::path::{Path, PathBuf};

/// This command can be used to run operations on DFAs or NFAs.
///
/// `--type dfa/nfa` specifies if you are testing DFAs or NFAs. Defaults to DFA.
/// `--condition all/any` specifies if all lines or just any lines of the files needs to be accepted by the DFA
///     when testing it.
/// The `equivalence` operation checks which of many automata are equivalent to the automata provided.
/// The `test` operation checks which files has all/any lines accepted by the automata.
// Example usage: dandy-cli equivalence tests/dfa1.dfa tests/example_tree/**/*.dfa
//                dandy-cli equivalence --in-type nfa --minimized tests/nfa1.nfa tests/example_tree/**/*.dfa
#[derive(Parser, Debug)]
#[command(
    version,
    author = "Jonathan Widén",
    about = "A cli tool for importing and checking DFAs and NFAs",
    long_about
)]
struct DandyArgs {
    #[arg(
        short,
        long = "out",
        help = "Outputs the result of the command into a file"
    )]
    out_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Disables additional logging (but still outputs result to stdout)",
        default_value_t
    )]
    no_log: bool,
    #[command(subcommand)]
    command: Operation,
}

#[derive(Debug, Subcommand)]
enum Operation {
    Equivalence(EquivalenceArgs),
    Union(UnionArgs),
    Intersection(IntersectionArgs),
}

#[derive(Debug, Args)]
struct EquivalenceArgs {
    #[arg(
        short,
        long,
        value_enum,
        help = "Choose the input type of the correct answer (defaults to the same as the test type)"
    )]
    in_type: Option<FaType>,
    #[arg(
        short,
        long,
        value_enum,
        default_value_t,
        help = "Choose if testing DFAs, NFAs or Regexes"
    )]
    r#type: FaType,
    //#[arg(
    //    short,
    //    long,
    //    value_enum,
    //    default_value_t,
    //    help = "(For 'test' operation): Choose if all lines or one line per file must be accepted by the automata"
    //)]
    //condition: TestType,
    //#[arg(
    //    help = "Choose if you want to check what files define automata equivalent to the given automata, or what files has lines accepted by the automata",
    //)]
    //operation: OpType,
    #[arg(
        short,
        long,
        default_value_t,
        help = "(For 'DFA' test type only): Requires the DFAs to be minimized"
    )]
    minimized: bool,
    #[arg(
        short,
        long,
        default_value_t,
        help = "Output 'true'/'false' rather than a result in text format"
    )]
    r#bool: bool,
    #[arg(long, default_value_t, help = "Disables additional logging")]
    no_log: bool,
    #[arg(short, long, help = "How many path components to print (0 to disable)")]
    path_length: Option<usize>,
    #[arg()]
    automata: PathBuf,
    #[arg()]
    files: Vec<PathBuf>,
}

#[derive(Debug, Args)]
struct UnionArgs {
    first_dfa: PathBuf,
    second_dfa: PathBuf,
}

#[derive(Debug, Args)]
struct IntersectionArgs {
    first_dfa: PathBuf,
    second_dfa: PathBuf,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum FaType {
    #[default]
    Dfa,
    Nfa,
    Regex,
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
    let args = DandyArgs::parse();
    match &args.command {
        Operation::Equivalence(eq_args) => {
            // We want to read the input file here to get enough lifetime for the error
            let file = fs::read_to_string(&eq_args.automata).unwrap();
            equivalence::equivalence(&args, eq_args, &file)
        }
        Operation::Union(union_args) => union::union(&args, union_args),
        Operation::Intersection(inters_args) => intersection::intersection(&args, inters_args),
    };

    /*let file = fs::read_to_string(&args.automata);
    match file {
        Err(e) => {
            eprintln!("Error reading input file: {e}");
        }
        Ok(f) => match args.operation {
            OpType::Equivalence => {
                if let Err(e) = equivalence::equivalence(&args, &f) {
                    eprintln!("Could not start test: {e}")
                }
            }
            OpType::Test => {
                test(&args, &f);
            }
        },
    }*/
}

pub fn last_n_components(path: &Path, n: Option<usize>) -> Option<String> {
    let Some(n) = n else {
        return Some(path.display().to_string());
    };
    (n != 0).then(|| {
        let new_path = path.components().rev().take(n).collect::<PathBuf>();
        new_path
            .components()
            .rev()
            .collect::<PathBuf>()
            .display()
            .to_string()
    })
}

enum Automata {
    Dfa(Dfa),
    Nfa(Nfa),
}

impl Automata {
    fn load_test(file: &str, r#type: FaType) -> Result<Self, EquivalenceResult> {
        match r#type {
            FaType::Dfa => {
                let dfa = parser::dfa(file)
                    .map_err(|e| EquivalenceResult::FailedToParse(e.to_string()))?
                    .try_into()
                    .map_err(|e: DfaParseError| {
                        EquivalenceResult::FailedToValidate(e.to_string())
                    })?;
                Ok(Automata::Dfa(dfa))
            }
            FaType::Nfa => {
                let nfa = parser::nfa(file)
                    .map_err(|e| EquivalenceResult::FailedToParse(e.to_string()))?
                    .try_into()
                    .map_err(|e: NfaParseError| {
                        EquivalenceResult::FailedToValidate(e.to_string())
                    })?;
                Ok(Automata::Nfa(nfa))
            }
            FaType::Regex => {
                let regex = parser::regex(file)
                    .map_err(|e| EquivalenceResult::FailedToParse(e.to_string()))?;
                let nfa = regex.to_nfa();
                Ok(Automata::Nfa(nfa)) // We don't really need to reduce states here as much, since
                                       // base testing with has fewer states
            }
        }
    }

    fn test_equivalence(&self, other: &Self, minimized: bool) -> EquivalenceResult {
        use equivalence::EquivalenceResult::*;
        match &self {
            Automata::Dfa(this_dfa) => {
                match other {
                    Automata::Dfa(other_dfa) => {
                        if this_dfa.equivalent_to(other_dfa) {
                            if this_dfa.states().len() == other_dfa.states().len() || !minimized {
                                Equivalent
                            } else {
                                NotMinimized
                            }
                        } else {
                            NotEquivalent
                        }
                    }
                    Automata::Nfa(other_nfa) => {
                        eprintln!("Comparing this (DFA) to other (NFA), performance inpact, shouldn't happen");
                        if minimized {
                            eprintln!("Trying to compare minimization of NFA, can't be done");
                        }
                        let this_nfa = this_dfa.clone().to_nfa();
                        if this_nfa.equivalent_to(other_nfa) {
                            Equivalent
                        } else {
                            NotEquivalent
                        }
                    }
                }
            }
            Automata::Nfa(this_nfa) => {
                match other {
                    Automata::Dfa(other_dfa) => {
                        eprintln!("Comparing this (NFA) to other (DFA), performance impact, shouldn't happen");
                        let mut this_dfa = this_nfa.to_dfa();
                        if minimized {
                            this_dfa.minimize();
                        }
                        if this_dfa.equivalent_to(other_dfa) {
                            if this_dfa.states().len() == other_dfa.states().len() || !minimized {
                                Equivalent
                            } else {
                                NotMinimized
                            }
                        } else {
                            NotEquivalent
                        }
                    }
                    Automata::Nfa(other_nfa) => {
                        if minimized {
                            eprintln!("Trying to compare minimization of NFA, can't be done");
                        }
                        if this_nfa.equivalent_to(other_nfa) {
                            Equivalent
                        } else {
                            NotEquivalent
                        }
                    }
                }
            }
        }
    }

    fn into_minimized_dfa(self) -> Self {
        let mut dfa = match self {
            Automata::Dfa(dfa) => dfa,
            Automata::Nfa(nfa) => nfa.to_dfa(),
        };
        dfa.minimize();
        Automata::Dfa(dfa)
    }

    fn prepare_to_compare_with(self, other: FaType) -> Self {
        match other {
            FaType::Dfa => self.into_dfa(),
            FaType::Nfa | FaType::Regex => self.into_nfa(),
        }
    }

    fn into_dfa(self) -> Self {
        match self {
            Automata::Dfa(dfa) => Automata::Dfa(dfa),
            Automata::Nfa(nfa) => Automata::Dfa(nfa.to_dfa()),
        }
    }

    fn into_nfa(self) -> Self {
        match self {
            Automata::Dfa(dfa) => Automata::Nfa(dfa.to_nfa()),
            Automata::Nfa(nfa) => Automata::Nfa(nfa),
        }
    }

    fn table(&self) -> String {
        match self {
            Automata::Dfa(dfa) => dfa.to_table(),
            Automata::Nfa(nfa) => nfa.to_table(),
        }
    }
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
    let dfa2 = parser::dfa(equivalent_dfa).unwrap().try_into().unwrap();
    assert!(dfa.equivalent_to(&dfa2));
}
