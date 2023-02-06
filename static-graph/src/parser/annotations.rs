use std::ops::Deref;

use nom::{
    bytes::complete::{tag, take_while},
    character::complete::satisfy,
    combinator::{map, opt, recognize},
    multi::many1,
    sequence::tuple,
    IResult,
};

use super::{blank, list_separator, literal::Literal, Parser};

#[derive(Debug, Clone)]
pub struct Annotation {
    pub key: String,
    pub value: Literal,
}

impl Deref for Annotations {
    type Target = Vec<Annotation>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Annotations(pub Vec<Annotation>);

impl Parser for Annotations {
    fn parse(input: &str) -> IResult<&str, Annotations> {
        map(
            tuple((
                tag("#["),
                many1(map(
                    tuple((
                        opt(blank),
                        recognize(tuple((
                            satisfy(|c| c.is_ascii_alphabetic() || c == '_'),
                            take_while(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.'),
                        ))),
                        opt(blank),
                        tag("="),
                        opt(blank),
                        Literal::parse,
                        opt(blank),
                        opt(list_separator),
                    )),
                    |(_, p, _, _, _, lit, _, _)| Annotation {
                        key: p.to_owned(),
                        value: lit,
                    },
                )),
                tag("]"),
            )),
            |(_, annotations, _)| Annotations(annotations),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotations() {
        match Annotations::parse(r#"#[foo = "bar"]"#) {
            Ok((remain, annotations)) => {
                assert_eq!(remain, "");
                assert_eq!(annotations.len(), 1);
                assert_eq!(annotations[0].key, "foo");
                assert_eq!(annotations[0].value.0, "bar");
            }
            Err(e) => panic!("{e:?}"),
        }
    }
}
