use crate::{DandyArgs, IntersectionArgs};
use dandy::dfa::Dfa;
use dandy::parser;
use std::fs;

pub fn intersection(_main_args: &DandyArgs, args: &IntersectionArgs) {
    let file1 = fs::read_to_string(&args.first_dfa).unwrap();
    let file2 = fs::read_to_string(&args.second_dfa).unwrap();
    let dfa1: Dfa = parser::dfa(&file1).unwrap().try_into().unwrap();
    let dfa2: Dfa = parser::dfa(&file2).unwrap().try_into().unwrap();
    match dfa1.intersection(&dfa2) {
        None => {
            println!("Alphabets differ: can't compute union")
        }
        Some(mut intersection) => {
            println!("Intersection:");
            println!("{}", intersection.to_table());

            intersection.minimize();
            println!("Minimized:");
            println!("{}", intersection.to_table());
        }
    }
}
