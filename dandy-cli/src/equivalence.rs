use crate::{Automata, DandyArgs, EquivalenceArgs, FaType};
use dandy::dfa::parse::DfaParseError;
use dandy::nfa::parse::NfaParseError;
use dandy::parser;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquivalenceResult {
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

pub fn equivalence<'a>(main_args: &DandyArgs, args: &EquivalenceArgs, file: &str) {
    run_equivalence(main_args, args, file).unwrap();
}

fn run_equivalence<'a>(
    _main_args: &DandyArgs,
    args: &EquivalenceArgs,
    file: &'a str,
) -> Result<(), Error<'a>> {
    let tester = DandyTester::new(&file, args)?;
    #[allow(unused_variables)]
    let log = |s: &str| {
        if !args.no_log {
            println!("{s}")
        }
    };
    macro_rules! log {
        ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
    }

    log!("Input loaded:");
    log!("{}", tester.input_automata().table());

    let start = SystemTime::now();
    let results = args
        .files
        .iter()
        .map(|path| (path, tester.test_equivalence(path)))
        .collect::<Vec<_>>();
    let duration = SystemTime::now().duration_since(start).unwrap_or_default();

    log!(
        "Testing of {} files done in {}ms. Results:",
        args.files.len(),
        duration.as_millis()
    );

    let successes = results.into_iter().fold(0usize, |acc, (path, result)| {
        let res = if args.bool {
            format!("{}", result == EquivalenceResult::Equivalent)
        } else {
            result.to_string()
        };
        if let Some(prefix) = crate::last_n_components(path, args.path_length) {
            println!("{prefix}: {res}");
        } else {
            println!("{res}");
        }

        if result == EquivalenceResult::Equivalent {
            acc + 1
        } else {
            acc
        }
    });

    log!("{}/{} files passed", successes, args.files.len());

    Ok(())
}

struct DandyTester {
    input: Automata,
    minimized: bool,
    test_type: FaType,
}

impl DandyTester {
    fn input_automata(&self) -> &Automata {
        &self.input
    }

    fn new<'a>(file: &'a str, args: &EquivalenceArgs) -> Result<DandyTester, Error<'a>> {
        let mut input = match args.in_type.unwrap_or(args.r#type) {
            FaType::Dfa => {
                let dfa = parser::dfa(file)
                    .map_err(Error::DfaParseError)?
                    .try_into()
                    .map_err(Error::DfaError)?;
                Automata::Dfa(dfa)
            }
            FaType::Nfa => {
                let nfa = parser::nfa(file)
                    .map_err(Error::NfaParseError)?
                    .try_into()
                    .map_err(Error::NfaError)?;
                Automata::Nfa(nfa)
            }
            FaType::Regex => {
                let regex = parser::regex(file).map_err(Error::RegexParseError)?;
                let nfa = regex.to_nfa();
                let dfa = nfa.to_dfa(); // To reduce states, regex->nfa can produce MANY states
                Automata::Dfa(dfa)
            }
        };

        let minimized = if args.minimized {
            if args.r#type == FaType::Dfa {
                input = input.into_minimized_dfa();
                true
            } else {
                return Err(Error::InvalidMinimizedConfig);
            }
        } else {
            false
        };

        input = input.prepare_to_compare_with(args.r#type);

        Ok(Self {
            input,
            minimized,
            test_type: args.r#type,
        })
    }

    fn test_equivalence(&self, file: &Path) -> EquivalenceResult {
        match fs::read_to_string(file) {
            Err(e) => EquivalenceResult::FailedToRead(e.to_string()),
            Ok(f) => match Automata::load_test(&f, self.test_type) {
                Ok(automata) => self.input.test_equivalence(&automata, self.minimized),
                Err(res) => res,
            },
        }
    }
}

#[derive(Error, Debug)]
enum Error<'a> {
    #[error("Error parsing DFA: {0:?}")]
    DfaParseError(nom::error::Error<&'a str>),
    #[error("Error compiling DFA: {0}")]
    DfaError(DfaParseError<'a>),
    #[error("Error parsing NFA: {0:?}")]
    NfaParseError(nom::error::Error<&'a str>),
    #[error("Error compiling NFA: {0}")]
    NfaError(NfaParseError<'a>),
    #[error("Error parsing regular expression: {0:?}")]
    RegexParseError(nom::error::Error<&'a str>),
    #[error("--minimized option can only be used when testing DFAs")]
    InvalidMinimizedConfig,
}
