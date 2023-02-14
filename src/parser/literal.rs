use std::ops::Deref;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag},
    character::complete::{none_of, one_of},
    combinator::map,
    sequence::delimited,
    IResult,
};

use super::Parser;

#[derive(Debug, Clone)]
pub struct Literal(pub String);

impl Deref for Literal {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl<'a> Parser<'a> for Literal {
    fn parse(input: &'a str) -> IResult<&'a str, Literal> {
        alt((
            map(single_quote, |x| Literal(x.into())),
            map(double_quote, |x| Literal(x.into())),
        ))(input)
    }
}

fn single_quote(input: &str) -> IResult<&str, &str> {
    let esc = escaped(none_of(r#"\'"#), '\\', one_of(r#"'"n\"#));
    let esc_or_empty = alt((esc, tag("")));
    let res = delimited(tag("\'"), esc_or_empty, tag("\'"))(input)?;

    Ok(res)
}

fn double_quote(input: &str) -> IResult<&str, &str> {
    let esc = escaped(none_of(r#"\""#), '\\', one_of(r#"'"n\"#));
    let esc_or_empty = alt((esc, tag("")));
    let res = delimited(tag("\""), esc_or_empty, tag("\""))(input)?;

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let input = r#""foo""#;
        match super::Literal::parse(input) {
            Ok((remain, lit)) => {
                assert_eq!(remain, "");
                assert_eq!(lit.0, "foo");
            }
            Err(e) => panic!("Error: {e:?}"),
        }
    }
}
