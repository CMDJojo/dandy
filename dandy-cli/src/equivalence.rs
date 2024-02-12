use crate::{automata::Automata, DandyArgs, EquivalenceArgs};
use dandy::dfa::parse::DfaParseError;
use dandy::nfa::parse::NfaParseError;
use dandy::parser;
use std::fmt::Display;
use std::path::Path;
use std::time::SystemTime;
use std::{fs, io};
use thiserror::Error;
use crate::automata::AutomataType;

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

pub fn equivalence(
    main_args: &DandyArgs,
    args: &EquivalenceArgs,
    mut output: impl FnMut(&str),
) -> Result<(), String> {
    let file = fs::read_to_string(&args.automata).map_err(|e| Error::InputFile(e).to_string())?;

    let tester = DandyTester::new(&file, args).map_err(|e| e.to_string())?;
    #[allow(unused_variables)]
    let log = |s: &str| {
        if !main_args.no_log {
            println!("{s}")
        }
    };
    macro_rules! log {
        ($($t:tt)*) => (log(&format!($($t)*)))
    }
    macro_rules! output {
        ($($t:tt)*) => (output(&format!($($t)*)))
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
            output!("{prefix}: {res}");
        } else {
            output!("{res}");
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
    test_type: AutomataType,
}

impl DandyTester {
    fn input_automata(&self) -> &Automata {
        &self.input
    }

    fn new<'a>(file: &'a str, args: &EquivalenceArgs) -> Result<DandyTester, Error<'a>> {
        let mut input = match args.in_type.unwrap_or(args.r#type) {
            AutomataType::Dfa => {
                let dfa = parser::dfa(file)
                    .map_err(Error::DfaParse)?
                    .try_into()
                    .map_err(Error::Dfa)?;
                Automata::Dfa(dfa)
            }
            AutomataType::Nfa => {
                let nfa = parser::nfa(file)
                    .map_err(Error::NfaParse)?
                    .try_into()
                    .map_err(Error::Nfa)?;
                Automata::Nfa(nfa)
            }
            AutomataType::Regex => {
                let regex = parser::regex(file).map_err(Error::RegexParse)?;
                let nfa = regex.to_nfa();
                let dfa = nfa.to_dfa(); // To reduce states, regex->nfa can produce MANY states
                Automata::Dfa(dfa)
            }
        };

        let minimized = if args.minimized {
            if args.r#type == AutomataType::Dfa {
                (input, _) = input.to_minimized_dfa_automata();
                true
            } else {
                return Err(Error::InvalidMinimizedConfig);
            }
        } else {
            false
        };

        (input, _) = input.prepare_to_compare_with(args.r#type);

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
                Ok(automata) => self.input.test_equivalence(automata, self.minimized),
                Err(res) => res,
            },
        }
    }
}

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("Error parsing DFA: {0:?}")]
    DfaParse(nom::error::Error<&'a str>),
    #[error("Error compiling DFA: {0}")]
    Dfa(DfaParseError<'a>),
    #[error("Error parsing NFA: {0:?}")]
    NfaParse(nom::error::Error<&'a str>),
    #[error("Error compiling NFA: {0}")]
    Nfa(NfaParseError<'a>),
    #[error("Error parsing regular expression: {0:?}")]
    RegexParse(nom::error::Error<&'a str>),
    #[error("--minimized option can only be used when testing DFAs")]
    InvalidMinimizedConfig,
    #[error("Error reading input file: {0}")]
    InputFile(#[from] io::Error),
}
