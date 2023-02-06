use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};

use super::{annotations::Annotations, blank, ident::Ident, list_separator, ty::Type, Parser};

#[derive(Debug, Clone)]
pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub annotations: Annotations,
}

impl Parser for Field {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                opt(Annotations::parse),
                opt(blank),
                Ident::parse,
                tag(":"),
                opt(blank),
                Type::parse,
                opt(blank),
                opt(list_separator),
            )),
            |(annotations, _, name, _, _, ty, _, _)| Field {
                name,
                ty,
                annotations: annotations.unwrap_or_default(),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ty::Type;

    #[test]
    fn test_field() {
        let input = r#"#[default = "Bar::new"]
        foo: Bar"#;
        match super::Field::parse(input) {
            Ok((remain, field)) => {
                assert_eq!(remain, "");
                assert_eq!(field.name.0, "foo");
                match field.ty {
                    Type::Path(path) => {
                        assert_eq!(path.segments.len(), 1);
                        assert_eq!(path.segments[0].0, "Bar");
                    }
                    _ => panic!("Expected Path"),
                }
                assert_eq!(field.annotations.len(), 1);
                assert_eq!(field.annotations[0].key, "default");
                assert_eq!(field.annotations[0].value.0, "Bar::new");
            }
            Err(e) => panic!("{e:?}"),
        }
    }
}
