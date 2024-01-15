fn main() {
    println!("Hello, world!");
    let dfa = dandy::parser::dfa(
        include_str!("example.dfa")
    );
    dbg!(&dfa);
    if let Ok((_, dfa)) = dfa {
        let actual_dfa = dandy::dfa::Dfa::try_from(dfa);
        dbg!(&actual_dfa);
        if let Err(e) = actual_dfa {
            println!("{}", e.to_string())
        }
    }

    let dfa1: dandy::dfa::Dfa = dandy::parser::dfa(
        include_str!("example.dfa")
    ).unwrap().1.try_into().unwrap();

    let dfa2: dandy::dfa::Dfa = dandy::parser::dfa(
        include_str!("example2.dfa")
    ).unwrap().1.try_into().unwrap();

    println!("DFA1!!!!");
    dbg!(&dfa1);
    println!("DFA2!!!!");
    dbg!(&dfa2);

    println!("{}", dfa1.equivalent_to(&dfa2));

    println!("{}", dfa2.to_table());


    let nfa = dandy::parser::nfa(
        include_str!("example.nfa")
    ).unwrap().1;
    let nfa: dandy::nfa::Nfa = nfa.try_into().unwrap();
    let output = nfa.to_table();
    println!("{output}");

    let nfa = dandy::parser::nfa(
        include_str!("example2.nfa")
    ).unwrap().1;
    let nfa: dandy::nfa::Nfa = nfa.try_into().unwrap();
    let output = nfa.to_table();
    println!("{output}");
}
