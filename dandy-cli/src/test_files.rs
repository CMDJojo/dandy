use crate::automata::Automata;
use crate::{DandyArgs, TestFileArgs, TestType};
use std::fs;

pub fn test_files(
    main_args: &DandyArgs,
    args: &TestFileArgs,
    #[allow(unused_variables, unused_mut)] mut output: impl FnMut(&str),
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

    let automata = Automata::load_file(&args.automata, args.r#type)?;
    let (nfa, _) = automata.into_nfa();
    log!("Loaded NFA:\n{}", nfa.to_table());

    for file in &args.files {
        let loaded_file = fs::read_to_string(file).map_err(|e| e.to_string())?;
        if args.test_type == TestType::Lines {
            output!("Testing file {}:", file.display());
            let mut n = 0;
            let mut a = 0;
            for line in loaded_file.lines() {
                n += 1;
                let accepted = nfa.accepts_graphemes(line);
                let ok = if accepted {
                    a += 1;
                    "[ OK ]"
                } else {
                    "[FAIL]"
                };
                output!("{ok} {line}");
            }
            output!("{a}/{n} lines passed in file {}:", file.display());
        } else {
            let counterexample = loaded_file
                .lines()
                .find(|line| !nfa.accepts_graphemes(line));
            match counterexample {
                None => {
                    output!("[ OK ] {}", file.display())
                }
                Some(c) => {
                    output!("[FAIL] {} failed on {c}", file.display())
                }
            }
        }
    }

    Ok(())
}
