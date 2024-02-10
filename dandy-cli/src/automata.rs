use crate::equivalence::EquivalenceResult;
use clap::ValueEnum;
use dandy::dfa::parse::DfaParseError;
use dandy::dfa::Dfa;
use dandy::nfa::parse::NfaParseError;
use dandy::nfa::Nfa;
use dandy::parser;
use dandy::regex::Regex;
use std::path::Path;
use std::{fs, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error("Error loading file {0}: {1}")]
    File(&'a Path, io::Error),
    #[error("Error parsing DFA: {0}")]
    DfaParse(nom::error::Error<&'a str>),
    #[error("Error compiling DFA: {0}")]
    DfaCompile(DfaParseError<'a>),
    #[error("Error parsing DFA: {0}")]
    NfaParse(nom::error::Error<&'a str>),
    #[error("Error compiling DFA: {0}")]
    NfaCompile(NfaParseError<'a>),
    #[error("Error parsing Regex: {0}")]
    RegexParse(nom::error::Error<&'a str>),
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum AutomataType {
    #[default]
    Dfa,
    Nfa,
    Regex,
}

impl AutomataType {
    pub fn to_string(self, multiple: bool) -> &'static str {
        match (self, multiple) {
            (AutomataType::Dfa, true) => "DFAs",
            (AutomataType::Dfa, false) => "DFA",
            (AutomataType::Nfa, true) => "NFAs",
            (AutomataType::Nfa, false) => "NFA",
            (AutomataType::Regex, true) => "Regexes",
            (AutomataType::Regex, false) => "Regex",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Automata {
    Dfa(Dfa),
    Nfa(Nfa),
    Regex(Regex),
}

#[allow(dead_code)]
impl Automata {
    /// Gets the type of the value contained in this Automata
    pub fn get_type(&self) -> AutomataType {
        match self {
            Automata::Dfa(_) => AutomataType::Dfa,
            Automata::Nfa(_) => AutomataType::Nfa,
            Automata::Regex(_) => AutomataType::Regex,
        }
    }

    /// Loads an automata of any type by reading and parsing it from a file.
    pub fn load_file(path: &Path, r#type: AutomataType) -> Result<Self, String> {
        let file = fs::read_to_string(path).map_err(|e| Error::File(path, e).to_string());
        file.and_then(|f| Self::load(&f, r#type).map_err(|e| e.to_string()))
    }

    /// Loads an automata of any type by parsing it from a string.
    pub fn load(file: &str, r#type: AutomataType) -> Result<Self, Error> {
        match r#type {
            AutomataType::Dfa => {
                let dfa: Dfa = parser::dfa(file)
                    .map_err(Error::DfaParse)?
                    .try_into()
                    .map_err(Error::DfaCompile)?;
                Ok(Self::Dfa(dfa))
            }
            AutomataType::Nfa => {
                let nfa: Nfa = parser::nfa(file)
                    .map_err(Error::NfaParse)?
                    .try_into()
                    .map_err(Error::NfaCompile)?;
                Ok(Self::Nfa(nfa))
            }
            AutomataType::Regex => parser::regex(file)
                .map(Self::Regex)
                .map_err(Error::RegexParse),
        }
    }

    /// Converts this Automata to a minimized DFA if the automata isn't already a DFA. If it is, nothing happens.
    /// Returns the DFA and a bool indicating whether a conversion occurred.
    pub fn to_minimized_dfa_if_not_dfa(self) -> (Dfa, bool) {
        let (mut dfa, converted) = self.to_dfa();
        if !converted {
            (dfa, false)
        } else {
            dfa.minimize();
            (dfa, true)
        }
    }

    /// Converts this Automata to a minimized DFA (independent of automata type). Returns the DFA and a bool indicating
    /// whether or not either a conversion or a minimization occurred.
    pub fn to_minimized_dfa(self) -> (Dfa, bool) {
        let (mut dfa, converted) = self.to_dfa();
        let prev_states = dfa.states().len();
        dfa.minimize();
        let new_states = dfa.states().len();
        (dfa, converted || new_states != prev_states)
    }

    /// Converts this Automata to a minimized DFA (independent of automata type) wrapped in this Automata enum. Returns
    /// the Automata which is guaranteed to be a DFA and a bool indicating whether or not either a conversion or a
    /// minimization occurred.
    pub fn to_minimized_dfa_automata(self) -> (Self, bool) {
        let (dfa, converted) = self.to_minimized_dfa();
        (Self::Dfa(dfa), converted)
    }

    /// Converts this Automata to a DFA (independent of automata type). Returns the DFA and a bool indicating
    /// whether or not a conversion occurred.
    pub fn to_dfa(self) -> (Dfa, bool) {
        match self {
            Automata::Dfa(dfa) => (dfa, false),
            Automata::Nfa(nfa) => (nfa.to_dfa(), true),
            Automata::Regex(regex) => (regex.to_nfa().to_dfa(), true),
        }
    }

    /// Converts this Automata to a DFA (independent of automata type) wrapped in this Automata enum. Returns the
    /// Automata which is guaranteed to be a DFA, and a bool indicating whether or not a conversion occurred.
    pub fn to_dfa_automata(self) -> (Self, bool) {
        let (dfa, converted) = self.to_dfa();
        (Self::Dfa(dfa), converted)
    }

    /// Borrows this Automata as a DFA, accessing the value within
    pub fn borrow_dfa(&self) -> Option<&Dfa> {
        match self {
            Automata::Dfa(dfa) => Some(dfa),
            _ => None,
        }
    }

    /// Converts this Automata to a NFA (independent of automata type). Returns the NFA and a bool indicating
    /// whether or not a conversion occurred.
    pub fn to_nfa(self) -> (Nfa, bool) {
        match self {
            Automata::Dfa(dfa) => (dfa.to_nfa(), true),
            Automata::Nfa(nfa) => (nfa, false),
            Automata::Regex(regex) => (regex.to_nfa(), true),
        }
    }

    /// Converts this Automata to a NFA (independent of automata type) wrapped in this Automata enum. Returns the
    /// Automata which is guaranteed to be a NFA, and a bool indicating whether or not a conversion occurred.
    pub fn to_nfa_automata(self) -> (Self, bool) {
        let (nfa, converted) = self.to_nfa();
        (Self::Nfa(nfa), converted)
    }

    /// Borrows this Automata as a DFA, accessing the value within
    pub fn borrow_nfa(&self) -> Option<&Nfa> {
        match self {
            Automata::Nfa(nfa) => Some(nfa),
            _ => None,
        }
    }

    /// Converts this Automata to a DFA or NFA. Note that the provided type cannot be Regex, if that is the case,
    /// this function returns None. Otherwise it returns the Automata and a bool indicating if a conversion
    /// occurred.
    pub fn convert_to(self, r#type: AutomataType) -> Option<(Self, bool)> {
        match r#type {
            AutomataType::Dfa => Some(self.to_dfa_automata()),
            AutomataType::Nfa => Some(self.to_nfa_automata()),
            AutomataType::Regex => {
                if let Self::Regex(regex) = self {
                    Some((Self::Regex(regex), false))
                } else {
                    eprintln!("Cannot convert DFA/NFA to Regex");
                    None
                }
            }
        }
    }

    /// Converts this Automata to the Automata type most appropriate to compare with the given Automata type. To compare
    /// this Automata to a DFA, the most appropriate type is another DFA, and to compare this Automata to a NFA or a
    /// Regex, the most appropriate type is a NFA. This speeds up equivalence checking later on.
    pub fn prepare_to_compare_with(self, other: AutomataType) -> (Self, bool) {
        match other {
            AutomataType::Dfa => self.to_dfa_automata(),
            AutomataType::Nfa | AutomataType::Regex => self.to_nfa_automata(),
        }
    }

    pub fn test_equivalence(&self, other: Self, minimized: bool) -> EquivalenceResult {
        macro_rules! warn_minimized_check_type {
            ($m:expr, $t:expr) => {
                if $t.get_type() == T::Dfa && $m {
                    eprintln!("Minimized option ignored: make sure the base automata is the correct type.");
                    eprintln!("This is most likely an internal error; please send a bug report")
                } else {
                    warn_minimized!($m);
                }
            };
        }

        macro_rules! warn_minimized {
            ($m:expr) => {
                if $m {
                    eprintln!("Can only check minimization if the tested type is a DFA")
                }
            };
        }

        use AutomataType as T;
        use EquivalenceResult::*;
        match (self.get_type(), other.get_type()) {
            (T::Dfa, T::Dfa) => {
                let dfa1 = self.borrow_dfa().unwrap();
                let dfa2 = other.borrow_dfa().unwrap();
                if dfa1.equivalent_to(dfa2) {
                    if minimized && dfa1.states().len() != dfa2.states().len() {
                        NotMinimized
                    } else {
                        Equivalent
                    }
                } else {
                    NotEquivalent
                }
            }
            (T::Dfa, _) => {
                warn_minimized!(minimized);
                let dfa1 = self.borrow_dfa().unwrap();
                let (dfa2, _) = other.to_dfa();
                if dfa1.equivalent_to(&dfa2) {
                    Equivalent
                } else {
                    NotEquivalent
                }
            }
            (T::Nfa, _) => {
                warn_minimized_check_type!(minimized, other);
                let nfa1 = self.borrow_nfa().unwrap();
                let (nfa2, _) = other.to_nfa();
                if nfa1.equivalent_to(&nfa2) {
                    Equivalent
                } else {
                    NotEquivalent
                }
            }
            (T::Regex, _) => {
                eprintln!("Testing with Regex as base, this gives poor performance");
                eprintln!("This is most likely an internal error; please send a bug report");
                warn_minimized!(minimized);
                let (dfa1, _) = self.clone().to_dfa();
                let (dfa2, _) = other.to_dfa();
                if dfa1.equivalent_to(&dfa2) {
                    Equivalent
                } else {
                    NotEquivalent
                }
            }
        }
    }
}

impl Automata {
    // TODO: Rewrite this
    pub fn load_test(file: &str, r#type: AutomataType) -> Result<Self, EquivalenceResult> {
        match r#type {
            AutomataType::Dfa => {
                let dfa = parser::dfa(file)
                    .map_err(|e| EquivalenceResult::FailedToParse(e.to_string()))?
                    .try_into()
                    .map_err(|e: DfaParseError| {
                        EquivalenceResult::FailedToValidate(e.to_string())
                    })?;
                Ok(Automata::Dfa(dfa))
            }
            AutomataType::Nfa => {
                let nfa = parser::nfa(file)
                    .map_err(|e| EquivalenceResult::FailedToParse(e.to_string()))?
                    .try_into()
                    .map_err(|e: NfaParseError| {
                        EquivalenceResult::FailedToValidate(e.to_string())
                    })?;
                Ok(Automata::Nfa(nfa))
            }
            AutomataType::Regex => {
                let regex = parser::regex(file)
                    .map_err(|e| EquivalenceResult::FailedToParse(e.to_string()))?;
                let nfa = regex.to_nfa();
                Ok(Automata::Nfa(nfa)) // We don't really need to reduce states here as much, since
                                       // base testing with has fewer states
            }
        }
    }

    pub fn table(&self) -> String {
        match self {
            Automata::Dfa(dfa) => dfa.to_table(),
            Automata::Nfa(nfa) => nfa.to_table(),
            Automata::Regex(regex) => regex.to_string(),
        }
    }
}
