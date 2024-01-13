use nom::{
    IResult,
    Parser,
    combinator::map,
    bytes::complete::tag,
    branch::alt,
};
use nom::bytes::complete::{take_till1};
use nom::character::complete::{line_ending, space0, space1};
use nom::combinator::opt;
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, terminated, tuple};

#[derive(Debug)]
pub struct ParsedDfa<'a> {
    pub head: Vec<&'a str>,
    pub states: Vec<ParsedDfaState<'a>>,
}

#[derive(Debug)]
pub struct ParsedDfaState<'a> {
    pub name: &'a str,
    pub initial: bool,
    pub accepting: bool,
    pub transitions: Vec<&'a str>,
}

pub fn dfa(input: &str) -> IResult<&str, ParsedDfa> {
    map(pair(
        terminated(
            dfa_head,
            line_ending,
        ),
        separated_list1(
            line_ending,
            dfa_line,
        ),
    ), |(head, states)|
            ParsedDfa { head, states })
        (input)
}

fn dfa_head(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        space0,
        separated_list1(
            space1,
            alphabet_char,
        ),
        space0,
    )(input)
}

fn dfa_line(input: &str) -> IResult<&str, ParsedDfaState> {
    map(tuple((
        space0,
        opt(arrow),
        space0,
        opt(accepting),
        space0,
        state_name,
        space1,
        separated_list1(
            space1,
            state_name,
        ),
        space0
    )),
        |(_, initial, _, accepting, _, name, _, transitions, _)|
            ParsedDfaState {
                name,
                initial: initial.is_some(),
                accepting: accepting.is_some(),
                transitions,
            })(input)
}

fn alphabet_char(input: &str) -> IResult<&str, &str> {
    take_till1(|c| " \t\n,{}".contains(c))(input)
}

fn state_name(input: &str) -> IResult<&str, &str> {
    take_till1(|c| " \t\n,{}".contains(c))(input)
}

fn accepting(input: &str) -> IResult<&str, ()> {
    map(
        tag("*"),
        |_| (),
    )(input)
}

fn arrow(input: &str) -> IResult<&str, ()> {
    map(alt((
        tag("->"),
        tag("→")
    )), |_| ())(input)
}