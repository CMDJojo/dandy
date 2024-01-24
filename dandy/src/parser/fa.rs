use crate::parser::{NfaAlphabetEntry, ParsedDfa, ParsedDfaState, ParsedNfa, ParsedNfaState};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1};
use nom::character::complete::{line_ending, not_line_ending, space0, space1};
use nom::combinator::{eof, map, opt, recognize, value, verify};
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

pub(crate) fn full_nfa(input: &str) -> IResult<&str, ParsedNfa> {
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

pub(crate) fn full_dfa(input: &str) -> IResult<&str, ParsedDfa> {
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
    // We need to allow a space-only or comment-only line to end with either
    // a line ending or eof, but we need to consume *something* otherwise
    // many0(space_comment_line) will be in an endless loop at eof
    value(
        (),
        verify(
            recognize(terminated(space_comment, alt((line_ending, eof)))),
            |consumed: &str| !consumed.is_empty(),
        ),
    )(input)
}

fn space_comment(input: &str) -> IResult<&str, ()> {
    value((), pair(space0, opt(comment)))(input)
}

fn comment(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("#"), not_line_ending))(input)
}
