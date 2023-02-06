use super::{blank, field::Field, ident::Ident, list_separator, Parser};

use nom::{
    bytes::complete::tag,
    combinator::map,
    combinator::opt,
    multi::many0,
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};

#[derive(Debug, Clone)]
pub struct Node {
    pub name: Ident,
    pub to_nodes: Vec<Ident>,
    pub fields: Vec<Field>,
}

impl Parser for Node {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                tag("node"),
                blank,
                Ident::parse,
                opt(blank),
                opt(map(
                    tuple((
                        tag("->"),
                        opt(blank),
                        delimited(
                            opt(tag("(")),
                            many0(map(
                                tuple((opt(blank), Ident::parse, opt(list_separator))),
                                |(_, to_ident, _)| to_ident,
                            )),
                            opt(tag(")")),
                        ),
                        opt(blank),
                    )),
                    |(_, _, to_idents, _)| to_idents,
                )),
                tag("{"),
                opt(blank),
                separated_list0(blank, Field::parse),
                opt(blank),
                tag("}"),
            )),
            |(_, _, name, _, to_nodes, _, _, fields, _, _)| Node {
                name,
                to_nodes: to_nodes.unwrap_or_default(),
                fields,
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ty::Type;

    #[test]
    fn test_node() {
        let input = r#"node Foo {
            #[default = "Bar::new"]
            foo: Bar,
        }"#;
        match super::Node::parse(input) {
            Ok((remain, node)) => {
                assert_eq!(remain, "");
                assert_eq!(node.name.0, "Foo");
                assert_eq!(node.to_nodes.len(), 0);
                assert_eq!(node.fields.len(), 1);
                assert_eq!(node.fields[0].name.0, "foo");
                match &node.fields[0].ty {
                    Type::Path(path) => {
                        assert_eq!(path.segments.len(), 1);
                        assert_eq!(path.segments[0].0, "Bar");
                    }
                    _ => panic!("Expected Path"),
                }
                assert_eq!(node.fields[0].annotations.len(), 1);
                assert_eq!(node.fields[0].annotations[0].key, "default");
            }
            Err(e) => panic!("Error: {e:?}"),
        }
    }
}
