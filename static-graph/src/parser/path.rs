use std::sync::Arc;

use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    multi::separated_list1,
    sequence::tuple,
    IResult,
};

use super::{blank, ident::Ident, Parser};

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Arc<[Ident]>,
}

impl Parser for Path {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            separated_list1(tuple((opt(blank), tag("::"), opt(blank))), Ident::parse),
            |idents| Path {
                segments: Arc::from(idents),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let input = "Foo::Bar::Baz";
        match super::Path::parse(input) {
            Ok((remain, path)) => {
                assert_eq!(remain, "");
                assert_eq!(path.segments.len(), 3);
                assert_eq!(path.segments[0].0, "Foo");
                assert_eq!(path.segments[1].0, "Bar");
                assert_eq!(path.segments[2].0, "Baz");
            }
            Err(e) => panic!("Error: {e:?}"),
        }
    }
}
