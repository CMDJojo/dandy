use crate::automata::Automata;
use crate::equivalence::EquivalenceResult;
use crate::{BinaryOpArgs, BinaryOperation, DandyArgs};
use thiserror::Error;

pub fn binary_op(
    main_args: &DandyArgs,
    args: &BinaryOpArgs,
    op: BinaryOperation,
    output: &mut impl FnMut(&str),
) -> Result<(), String> {
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

    let (mut dfa1, converted1) = Automata::load_file(&args.first, args.r#type)
        .map_err(|e| Error::InputFile(1, e).to_string())?
        .to_minimized_dfa_if_not_dfa();
    let (mut dfa2, converted2) =
        Automata::load_file(&args.second, args.second_type.unwrap_or(args.r#type))
            .map_err(|e| Error::InputFile(2, e).to_string())?
            .to_minimized_dfa_if_not_dfa();

    if converted1 {
        log!("Input file 1 was converted to a minimized DFA to proceed, since it wasn't a DFA to start with");
    } else if args.minimized {
        dfa1.minimize();
        log!("Minimized DFA 1 before doing product construction");
    }

    if converted2 {
        log!("Input file 2 was converted to a minimized DFA to proceed, since it wasn't a DFA to start with");
    } else if args.minimized {
        dfa2.minimize();
        log!("Minimized DFA 2 before doing product construction");
    }

    let combined = match op {
        BinaryOperation::Union => dfa1.union(&dfa2),
        BinaryOperation::Intersection => dfa1.intersection(&dfa2),
        BinaryOperation::Difference => dfa1.difference(&dfa2),
        BinaryOperation::SymmetricDifference => dfa1.symmetric_difference(&dfa2),
    };

    let Some(mut combined) = combined else {
        return Err(Error::DifferentAlphabets.to_string());
    };

    if args.minimized {
        combined.minimize();
        log!(
            "Minimized DFA ({} of the two provided {}):",
            op.as_str(),
            args.r#type.to_string(true)
        );
        output!("{}", combined.to_table());
    } else {
        log!(
            "{} of {} (not minimized, add --minimized to minimize):",
            op.as_str(),
            args.r#type.to_string(true)
        );
        output!("{}", combined.to_table());
    }

    if let Some(n) = args.generate {
        println!("First {n} words of the {}:", op.as_str().to_lowercase());
        let mut x = 0;
        combined.clone().to_nfa().words().take(n).for_each(|word| {
            if word.is_empty() {
                println!("(empty word)");
            } else {
                println!("{word}");
            }
            x += 1;
        });
        if x != n {
            println!("(only {x} words exists in the {})", op.as_str().to_lowercase());
        }
    }

    if let Some(path) = &args.compare_against {
        // We load the other DFA and then check equivalence to this DFA
        let compare_to = Automata::load_file(path, args.compared_type)
            .map_err(|e| Error::CompareTo(e).to_string())?;
        let result = match Automata::Dfa(combined).test_equivalence(compare_to, false) {
            EquivalenceResult::Equivalent => "EQUIVALENT",
            _ => "NOT EQUIVALENT",
        };
        output!(
            "{} of the two provided {} is {} to the third {}",
            op,
            args.r#type.to_string(true),
            result,
            args.compared_type.to_string(false)
        );
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Different alphabets in input DFAs, can't do product construction")]
    DifferentAlphabets,
    #[error("Error comparing with automata: {0}")]
    CompareTo(String),
    #[error("Error reading {0}: {1}")]
    InputFile(usize, String),
}
