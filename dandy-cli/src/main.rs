mod automata;
mod binary_op;
mod equivalence;
mod test_files;

use automata::AutomataType;
use clap::{Args, Parser, Subcommand, ValueEnum};
use dandy::dfa::Dfa;
use dandy::parser;
use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// A cli tool for parsing and checking DFAs, NFAs and Regexes.
// Example usage: dandy-cli equivalence tests/dfa1.dfa tests/example_tree/**/*.dfa
//                dandy-cli equivalence --in-type nfa --minimized tests/nfa1.nfa tests/example_tree/**/*.dfa
#[derive(Parser, Debug)]
#[command(version, author = "Jonathan Widén", about)]
struct DandyArgs {
    #[arg(
        short,
        long = "out",
        help = "Outputs the result of the command into a file"
    )]
    out_file: Option<PathBuf>,
    #[arg(
        long = "less-logs",
        help = "Disables additional logging (but still outputs result to stdout)",
        default_value_t
    )]
    no_log: bool,
    #[command(subcommand)]
    command: Operation,
}

#[derive(Debug, Subcommand)]
enum Operation {
    #[command(
        about = "Checks the equivalence of two or more automatas or regexes (if they define the same language)"
    )]
    Equivalence(EquivalenceArgs),
    #[command(
        about = "Computes the union of two automatas or regexes by conversion to DFA and product construction"
    )]
    Union(BinaryOpArgs),
    #[command(
        about = "Computes the intersection of two automatas or regexes by conversion to DFA and product construction"
    )]
    Intersection(BinaryOpArgs),
    #[command(
        about = "Computes the difference of two automatas or regexes by conversion to DFA and product construction"
    )]
    Difference(BinaryOpArgs),
    #[command(
        about = "Computes the symmetric difference of two automatas or regexes by conversion to DFA and product construction"
    )]
    SymmetricDifference(BinaryOpArgs),
    #[command(about = "Tests a list of files against an automata or regex")]
    TestFile(TestFileArgs),
}

#[derive(Debug, Args)]
struct TestFileArgs {
    #[arg(
        long,
        value_enum,
        default_value_t = AutomataType::Dfa,
        help = "The type of the automata/regex to test"
    )]
    r#type: AutomataType,
    #[arg(
        short,
        long,
        value_enum,
        default_value_t,
        help = "The way to interpret the input file, \
        either `lines` for treating each line is a separate test, or \
        `files` to accept each file depending if all lines match"
    )]
    test_type: TestType,
    #[arg(help = "The path to the automata or regex to test")]
    automata: PathBuf,
    #[arg(help = "The files to test")]
    files: Vec<PathBuf>
}

#[derive(Default, Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
enum TestType {
    #[default]
    Lines,
    Files,
}

impl Operation {
    fn binary_operation(&self) -> Option<BinaryOperation> {
        use BinaryOperation::*;
        match self {
            Operation::Union(_) => Some(Union),
            Operation::Intersection(_) => Some(Intersection),
            Operation::Difference(_) => Some(Difference),
            Operation::SymmetricDifference(_) => Some(SymmetricDifference),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BinaryOperation {
    Union,
    Intersection,
    Difference,
    SymmetricDifference,
}

impl BinaryOperation {
    fn as_str(&self) -> &'static str {
        match self {
            BinaryOperation::Union => "Union",
            BinaryOperation::Intersection => "Intersection",
            BinaryOperation::Difference => "Difference",
            BinaryOperation::SymmetricDifference => "Symmetric difference",
        }
    }

    fn as_str_lower(&self) -> &'static str {
        match self {
            BinaryOperation::Union => "union",
            BinaryOperation::Intersection => "intersection",
            BinaryOperation::Difference => "difference",
            BinaryOperation::SymmetricDifference => "symmetric difference",
        }
    }
}

impl fmt::Display for BinaryOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Args)]
struct EquivalenceArgs {
    #[arg(
        short,
        long,
        value_enum,
        help = "Choose the input type of the correct answer (defaults to the same as the test type)"
    )]
    in_type: Option<AutomataType>,
    #[arg(
        short,
        long,
        value_enum,
        default_value_t,
        help = "Choose if testing DFAs, NFAs or Regexes"
    )]
    r#type: AutomataType,
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
        help = "(Only for testing 'DFA's): Requires the DFAs to be minimized"
    )]
    minimized: bool,
    #[arg(
        short,
        long,
        default_value_t,
        help = "Output 'true'/'false' rather than a result in text format"
    )]
    r#bool: bool,
    #[arg(short, long, help = "How many path components to print (0 to disable)")]
    path_length: Option<usize>,
    #[arg(help = "The main automata to compare the other automatas to")]
    automata: PathBuf,
    #[arg(help = "Other files containing automata to compare to the main automata")]
    files: Vec<PathBuf>,
}

#[derive(Debug, Args)]
struct BinaryOpArgs {
    #[arg(
        short,
        long,
        value_enum,
        default_value_t = AutomataType::Dfa,
        help = "The type of the automatas to operate on"
    )]
    r#type: AutomataType,
    #[arg(
        short,
        long,
        help = "The type of the second automata to operate on (if different to the first automata)"
    )]
    second_type: Option<AutomataType>,
    #[arg(
        long,
        value_enum,
        default_value_t = AutomataType::Dfa,
        help = "The type of the automata you are using to compare the result to"
    )]
    compared_type: AutomataType,
    #[arg(
        short,
        long,
        default_value_t,
        help = "Output the product construction as a minimized DFA"
    )]
    minimized: bool,
    #[arg(
        short,
        long,
        help = "Generates `n` strings of the resulting product construction"
    )]
    generate: Option<usize>,
    #[arg(help = "The first automata or regex to do the operation on")]
    first: PathBuf,
    #[arg(help = "The second automata or regex to do the operation on")]
    second: PathBuf,
    #[arg(help = "Optionally an automata or regex to compare the result of the operation to")]
    compare_against: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OpType {
    Equivalence,
    Test,
}

fn main() {
    let args = DandyArgs::parse();

    let mut out_file = args
        .out_file
        .as_ref()
        .and_then(|path| match File::create(path) {
            Ok(f) => Some(f),
            Err(e) => {
                eprintln!("Error creating output file {e}");
                eprintln!("Execution will continue but no data will be written to file");
                None
            }
        });

    let mut sink = |s: &str| {
        println!("{s}");
        if let Some(f) = out_file.as_mut() {
            f.write_all(s.as_bytes()).unwrap();
        };
    };

    let result = match &args.command {
        Operation::Equivalence(eq_args) => {
            equivalence::equivalence(&args, eq_args, &mut sink).map_err(Error::Equivalence)
        }
        Operation::Union(bin_args)
        | Operation::Intersection(bin_args)
        | Operation::Difference(bin_args)
        | Operation::SymmetricDifference(bin_args) => {
            let operation = args.command.binary_operation().unwrap();
            binary_op::binary_op(&args, bin_args, operation, &mut sink)
                .map_err(|e| Error::Binary(operation, e))
        }
        Operation::TestFile(test_args) => {
            test_files::test_files(&args, test_args, &mut sink).map_err(Error::TestFile)
        }
    };

    if let Err(e) = result {
        eprintln!("{e}");
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error("Error in Equivalence: {0}")]
    Equivalence(String),
    #[error("Error in {0}: {1}")]
    Binary(BinaryOperation, String),
    #[error("Error in testing file: {0}")]
    TestFile(String),
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
