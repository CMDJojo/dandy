use crate::regex::{Regex, RegexChar, RegexTree};
use nom::branch::alt;
use nom::bytes::complete::take;
use nom::character::complete;
use nom::character::complete::{one_of};
use nom::combinator::{map, opt, value, verify};
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, preceded};
use nom::{IResult, Parser};
use std::hint::unreachable_unchecked;

pub(crate) fn full_regex(input: &str) -> IResult<&str, Regex> {
    map(expression, |tree| {
        Regex { tree }
    })(input.trim()) //trim instead of delimited since otherwise trailing w.s. can be counted as tokens
}

fn expression(input: &str) -> IResult<&str, RegexTree> {
    alternation(input)
}

fn alternation(input: &str) -> IResult<&str, RegexTree> {
    map(
        separated_list1(complete::char('|'), sequence),
        wrap_multiple(RegexTree::Alt),
    )(input)
}

fn sequence(input: &str) -> IResult<&str, RegexTree> {
    map(
        many1(alt((par_expr, combinated_char))),
        wrap_multiple(RegexTree::Sequence),
    )(input)
}

fn wrap_multiple<T>(f: impl Fn(Vec<T>) -> T) -> impl Fn(Vec<T>) -> T {
    move |mut items| {
        if items.len() > 1 {
            f(items)
        } else {
            items.remove(0)
        }
    }
}

fn par_expr(input: &str) -> IResult<&str, RegexTree> {
    map(
        delimited(complete::char('('), expression, complete::char(')')).and(opt(one_of("+*"))),
        apply_kleene,
    )(input)
}

fn combinated_char(input: &str) -> IResult<&str, RegexTree> {
    map(
        map(regex_char, RegexTree::Char).and(opt(one_of("+*"))),
        apply_kleene,
    )(input)
}

fn apply_kleene((to_combine, kleene): (RegexTree, Option<char>)) -> RegexTree {
    match kleene {
        Some('+') => RegexTree::Sequence(vec![
            to_combine.clone(),
            RegexTree::Repeat(Box::new(to_combine)),
        ]),
        Some('*') => RegexTree::Repeat(Box::new(to_combine)),
        None => to_combine,
        _ => unreachable!("Should only be +, * or none"),
    }
}

fn regex_char(input: &str) -> IResult<&str, RegexChar> {
    alt((empty_lang, empty_str, escaped_char, normal_char))(input)
}

fn normal_char(input: &str) -> IResult<&str, RegexChar> {
    verify(map(take(1usize), one_char), |rxc| match rxc {
        RegexChar::Char(c) => !is_reserved_char(*c),
        // Safety: mapped under one_char, it can only yield RegexChar::Char
        _ => unsafe { unreachable_unchecked() },
    })(input)
}

fn escaped_char(input: &str) -> IResult<&str, RegexChar> {
    map(preceded(complete::char('\\'), take(1usize)), one_char)(input)
}

// Helper function to take a &str of one char and return it as a RegexChar, panicking in debug
// mode upon multiple chars
fn one_char(c: &str) -> RegexChar {
    let mut chars = c.chars();
    let char = chars.next().expect("Take should take 1 char");
    #[cfg(debug_assertions)]
    assert_eq!(chars.next(), None, "take(1) should take only 1 character");
    RegexChar::Char(char)
}

fn empty_str(input: &str) -> IResult<&str, RegexChar> {
    value(RegexChar::Epsilon, complete::char('ε'))(input)
}
fn empty_lang(input: &str) -> IResult<&str, RegexChar> {
    value(RegexChar::Empty, complete::char('∅'))(input)
}

fn is_reserved_char(char: char) -> bool {
    ['(', ')', '∅', 'ε', '|', '*', '+', '\\'].contains(&char)
}
