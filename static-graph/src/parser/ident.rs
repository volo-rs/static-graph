use faststr::FastStr;
use nom::{
    bytes::complete::take_while,
    character::complete::{char, satisfy},
    combinator::{map, recognize},
    multi::many0,
    sequence::tuple,
    IResult,
};

use std::ops::Deref;

use super::Parser;

#[derive(Debug, Clone)]
pub struct Ident(pub FastStr);

impl Deref for Ident {
    type Target = FastStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parser for Ident {
    fn parse(input: &str) -> IResult<&str, Ident> {
        map(
            recognize(tuple((
                many0(char('_')),
                satisfy(|c| c.is_ascii_alphabetic()),
                take_while(|c: char| c.is_ascii_alphanumeric() || c == '_'),
            ))),
            |ident: &str| -> Ident { Ident(ident.into()) },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ident() {
        let input = "_Foo";
        match super::Ident::parse(input) {
            Ok((remain, ident)) => {
                assert_eq!(remain, "");
                assert_eq!(ident.0, "_Foo");
            }
            Err(e) => panic!("Error: {e:?}"),
        }
    }
}
