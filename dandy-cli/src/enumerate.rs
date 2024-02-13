use crate::automata::Automata;
use crate::{DandyArgs, EnumerateFileArgs, EnumerateRegexArgs};
use dandy::nfa::Nfa;
use dandy::parser;

pub fn enumerate_regex(
    main_args: &DandyArgs,
    args: &EnumerateRegexArgs,
    output: impl FnMut(&str),
) -> Result<(), String> {
    let regex = parser::regex(&args.regex).map_err(|e| e.to_string())?;
    let nfa = regex.to_nfa();
    enumerate_nfa(nfa, main_args, args.amount, output);
    Ok(())
}

pub fn enumerate_file(
    main_args: &DandyArgs,
    args: &EnumerateFileArgs,
    output: impl FnMut(&str),
) -> Result<(), String> {
    let file = Automata::load_file(&args.file, args.r#type)?;
    let (nfa, _) = file.into_nfa();
    enumerate_nfa(nfa, main_args, args.amount, output);
    Ok(())
}

fn enumerate_nfa(
    mut nfa: Nfa,
    main_args: &DandyArgs,
    n: usize,
    #[allow(unused_variables, unused_mut)] mut output: impl FnMut(&str),
) {
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

    nfa.remove_epsilon_moves();

    log!("First {n} words of the language of the regex:");
    let mut x = 0;
    nfa.words().take(n).for_each(|word| {
        if word.is_empty() {
            output!("(empty word)");
        } else {
            output!("{word}");
        }
        x += 1;
    });
    if x != n {
        log!("(only {x} words exists in the language of the regex)");
    }
}
