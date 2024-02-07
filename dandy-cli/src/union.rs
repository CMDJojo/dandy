use crate::{DandyArgs, UnionArgs};
use dandy::dfa::Dfa;
use dandy::parser;
use std::fs;

pub fn union(_main_args: &DandyArgs, args: &UnionArgs) {
    let file1 = fs::read_to_string(&args.first_dfa).unwrap();
    let file2 = fs::read_to_string(&args.second_dfa).unwrap();
    let dfa1: Dfa = parser::dfa(&file1).unwrap().try_into().unwrap();
    let dfa2: Dfa = parser::dfa(&file2).unwrap().try_into().unwrap();
    match dfa1.union(&dfa2) {
        None => {
            println!("Alphabets differ: can't compute union")
        }
        Some(mut union) => {
            println!("Union:");
            println!("{}", union.to_table());

            union.minimize();
            println!("Minimized:");
            println!("{}", union.to_table());
        }
    }
}
