use std::sync::Arc;

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};

use super::{blank, list_separator, path::Path, Parser};

#[derive(Debug, Clone)]
pub enum Type {
    String,
    Void,
    Byte,
    Bool,
    Binary,
    I8,
    I16,
    I32,
    I64,
    Double,
    List { value: Arc<Type> },
    Set { value: Arc<Type> },
    Map { key: Arc<Type>, value: Arc<Type> },
    Path(Path),
}

impl Parser for Type {
    fn parse(input: &str) -> IResult<&str, Self> {
        alt((
            map(tag("string"), |_| Type::String),
            map(tag("void"), |_| Type::Void),
            map(tag("byte"), |_| Type::Byte),
            map(tag("bool"), |_| Type::Bool),
            map(tag("binary"), |_| Type::Binary),
            map(tag("i8"), |_| Type::I8),
            map(tag("i16"), |_| Type::I16),
            map(tag("i32"), |_| Type::I32),
            map(tag("i64"), |_| Type::I64),
            map(tag("double"), |_| Type::Double),
            map(
                tuple((
                    tag("list"),
                    opt(blank),
                    tag("<"),
                    opt(blank),
                    Type::parse,
                    opt(blank),
                    tag(">"),
                )),
                |(_, _, _, _, inner_type, _, _)| Type::List {
                    value: Arc::new(inner_type),
                },
            ),
            map(
                tuple((
                    tag("set"),
                    opt(blank),
                    tag("<"),
                    opt(blank),
                    Type::parse,
                    opt(blank),
                    tag(">"),
                )),
                |(_, _, _, _, inner_type, _, _)| Type::Set {
                    value: Arc::new(inner_type),
                },
            ),
            map(
                tuple((
                    tag("map"),
                    opt(blank),
                    tag("<"),
                    opt(blank),
                    Type::parse,
                    opt(blank),
                    list_separator,
                    opt(blank),
                    Type::parse,
                    opt(blank),
                    tag(">"),
                )),
                |(_, _, _, _, key_type, _, _, _, value_type, _, _)| Type::Map {
                    key: Arc::new(key_type),
                    value: Arc::new(value_type),
                },
            ),
            map(Path::parse, Type::Path),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type() {
        let input = "string";
        match super::Type::parse(input) {
            Ok((remain, ty)) => {
                assert_eq!(remain, "");
                match ty {
                    Type::String => {}
                    _ => panic!("Expected String"),
                }
            }
            Err(e) => panic!("Error: {e:?}"),
        }
    }
}
