use dandy::dfa::Dfa;

fn main1() {
    let raw_dfa = "
           a  b  c
    → * s₀ s₁ s₀ s₂
        s₁ s₂ s₁ s₁
      * s₂ s₂ s₂ s₂
    ";
    let parsed_dfa = dandy::parser::dfa(raw_dfa).unwrap();
    let dfa: Dfa = parsed_dfa.try_into().unwrap();
    assert!(dfa.accepts(&vec!["a", "b", "c", "c", "a"]));
    assert!(dfa.accepts(&vec!["c", "b", "a"]));
    assert!(!dfa.accepts(&vec!["a", "b", "b", "c"]));

    let equivalent_dfa = "
        a b c
    → * x z x y
      * y y y y
        z y w z
        w y z w
    ";
    let dfa2 = dandy::parser::dfa(equivalent_dfa).unwrap().try_into().unwrap();
    assert!(dfa.equivalent_to(&dfa2));
}

fn main() {
    println!("Hello, world!");
    let dfa = dandy::parser::dfa(include_str!("example.dfa"));
    dbg!(&dfa);
    if let Ok(dfa) = dfa {
        let actual_dfa = dandy::dfa::Dfa::try_from(dfa);
        dbg!(&actual_dfa);
        if let Err(e) = actual_dfa {
            println!("{}", e)
        }
    }

    let dfa1: dandy::dfa::Dfa = dandy::parser::dfa(include_str!("example.dfa"))
        .unwrap()
        .try_into()
        .unwrap();

    let dfa2: dandy::dfa::Dfa = dandy::parser::dfa(include_str!("example2.dfa"))
        .unwrap()
        .try_into()
        .unwrap();

    println!("DFA1!!!!");
    dbg!(&dfa1);
    println!("DFA2!!!!");
    dbg!(&dfa2);

    println!("{}", dfa1.equivalent_to(&dfa2));

    println!("{}", dfa2.to_table());

    let nfa = dandy::parser::full_nfa(include_str!("example.nfa")).unwrap().1;
    let nfa: dandy::nfa::Nfa = nfa.try_into().unwrap();
    let output = nfa.to_table();
    println!("{output}");

    let nfa2 = dandy::parser::nfa(include_str!("example2.nfa")).unwrap();
    let nfa2: dandy::nfa::Nfa = nfa2.try_into().unwrap();
    let output = nfa2.to_table();
    println!("{output}");

    let nfa3 = dandy::parser::nfa(include_str!("example3.nfa")).unwrap();
    let nfa3: dandy::nfa::Nfa = nfa3.try_into().unwrap();
    let output = nfa3.to_table();
    println!("{output}");

    println!("{}", dfa1.equivalent_to(&dfa2));
    println!("{}", nfa2.equivalent_to(&nfa3));

    println!("{}", dfa2.to_table());
    println!("{}", dfa2.to_nfa().to_table());

    println!("Converted: (power construction)");
    println!("{}", nfa2.to_dfa().to_table());

    let eq_nfa_dfa = dandy::parser::dfa(include_str!("eq_example2_nfa.dfa"))
        .unwrap();
    let eq_nfa_dfa: dandy::dfa::Dfa = eq_nfa_dfa.try_into().unwrap();
    println!("Other:");
    println!("{}", eq_nfa_dfa.to_table());

    println!("Equivalent: {}", eq_nfa_dfa.equivalent_to(&nfa2.to_dfa()))
}
