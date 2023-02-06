pub mod annotations;
pub mod document;
pub mod field;
pub mod graph;
pub mod ident;
pub mod literal;
pub mod node;
pub mod path;
pub mod ty;

use nom::character::complete::{multispace1, one_of};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_until},
    combinator::map,
    multi::many1,
    sequence::{preceded, terminated},
    IResult,
};

pub trait Parser: Sized {
    fn parse(input: &str) -> IResult<&str, Self>;
}

fn comment(input: &str) -> IResult<&str, &str> {
    alt((
        preceded(tag("//"), take_till(|c| c == '\n')),
        preceded(tag("/*"), terminated(take_until("*/"), tag("*/"))),
    ))(input)
}

pub(crate) fn blank(input: &str) -> IResult<&str, ()> {
    map(many1(alt((comment, multispace1))), |_| ())(input)
}

pub(crate) fn list_separator(input: &str) -> IResult<&str, char> {
    one_of(",;")(input)
}
