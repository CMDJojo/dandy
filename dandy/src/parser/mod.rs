use nom::{branch::alt, bytes::complete::tag, bytes::complete::take_till1, character::complete::{line_ending, not_line_ending, space0, space1}, combinator::map, combinator::{opt, value, verify}, multi::{many0, many1, separated_list0, separated_list1}, sequence::{delimited, pair, preceded, terminated, tuple}, IResult, Finish};
use nom::error::Error;

#[derive(Debug)]
pub struct ParsedNfa<'a> {
    pub head: Vec<NfaAlphabetEntry<'a>>,
    pub states: Vec<ParsedNfaState<'a>>,
}

#[derive(Debug, Clone)]
pub enum NfaAlphabetEntry<'a> {
    Element(&'a str),
    Eps,
}

#[derive(Debug)]
pub struct ParsedNfaState<'a> {
    pub name: &'a str,
    pub initial: bool,
    pub accepting: bool,
    pub transitions: Vec<Vec<&'a str>>,
}

pub fn nfa(input: &str) -> Result<ParsedNfa, Error<&str>> {
    full_nfa(input).finish().map(|(_, nfa)| nfa)
}

pub fn full_nfa(input: &str) -> IResult<&str, ParsedNfa> {
    map(
        delimited(
            many0(space_comment_line),
            pair(
                terminated(nfa_head, line_ending),
                preceded(
                    many0(space_comment_line),
                    separated_list1(many1(space_comment_line), nfa_line),
                ),
            ),
            many0(space_comment_line),
        ),
        |(head, states)| ParsedNfa { head, states },
    )(input)
}

fn nfa_head(input: &str) -> IResult<&str, Vec<NfaAlphabetEntry>> {
    delimited(
        space0,
        separated_list1(
            space1,
            alt((
                map(alphabet_elem, NfaAlphabetEntry::Element),
                value(NfaAlphabetEntry::Eps, eps),
            )),
        ),
        space_comment,
    )(input)
}

fn nfa_line(input: &str) -> IResult<&str, ParsedNfaState> {
    map(
        delimited(
            space0,
            tuple((
                opt(
                    terminated(arrow, space1), // note: to be more lenient, change this and accepting to space0
                ),
                opt(terminated(accepting, space1)),
                terminated(state_name, space1),
                separated_list1(space1, state_set),
            )),
            space_comment,
        ),
        |(initial, accepting, name, transitions)| ParsedNfaState {
            name,
            initial: initial.is_some(),
            accepting: accepting.is_some(),
            transitions,
        },
    )(input)
}

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

pub fn dfa(input: &str) -> Result<ParsedDfa, Error<&str>> {
    full_dfa(input).finish().map(|(_, dfa)| dfa)
}

pub fn full_dfa(input: &str) -> IResult<&str, ParsedDfa> {
    map(
        delimited(
            many0(space_comment_line),
            pair(
                terminated(dfa_head, line_ending),
                preceded(
                    many0(space_comment_line),
                    separated_list1(many1(space_comment_line), dfa_line),
                ),
            ),
            many0(space_comment_line),
        ),
        |(head, states)| ParsedDfa { head, states },
    )(input)
}

fn dfa_head(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        space0,
        separated_list1(space1, alphabet_elem),
        space_comment,
    )(input)
}

fn dfa_line(input: &str) -> IResult<&str, ParsedDfaState> {
    map(
        delimited(
            space0,
            tuple((
                opt(
                    terminated(arrow, space1), // note: to be more lenient, change this and accepting to space0
                ),
                opt(terminated(accepting, space1)),
                terminated(state_name, space1),
                separated_list1(space1, state_name),
            )),
            space_comment,
        ),
        |(initial, accepting, name, transitions)| ParsedDfaState {
            name,
            initial: initial.is_some(),
            accepting: accepting.is_some(),
            transitions,
        },
    )(input)
}

fn eps(input: &str) -> IResult<&str, ()> {
    map(alt((tag("ε"), tag("eps"))), |_| ())(input)
}

fn alphabet_elem(input: &str) -> IResult<&str, &str> {
    verify(
        take_till1(|c: char| c.is_whitespace() || "#{}".contains(c)),
        |elem| !["ε", "eps", "→", "->", "*"].contains(&elem),
    )(input)
}

fn state_set(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(tag("{"), separated_list0(space1, state_name), tag("}"))(input)
}

fn state_name(input: &str) -> IResult<&str, &str> {
    verify(
        take_till1(|c: char| c.is_whitespace() || "#{}".contains(c)),
        |elem| !["ε", "eps", "→", "->", "*"].contains(&elem),
    )(input)
}

fn accepting(input: &str) -> IResult<&str, ()> {
    value((), tag("*"))(input)
}

fn arrow(input: &str) -> IResult<&str, ()> {
    map(alt((tag("->"), tag("→"))), |_| ())(input)
}

fn space_comment_line(input: &str) -> IResult<&str, ()> {
    terminated(space_comment, line_ending)(input)
}

fn space_comment(input: &str) -> IResult<&str, ()> {
    value((), pair(space0, opt(comment)))(input)
}

fn comment(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("#"), not_line_ending))(input)
}
