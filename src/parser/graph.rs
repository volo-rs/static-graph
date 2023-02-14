use nom::{bytes::complete::tag, combinator::map, sequence::tuple, IResult};

use super::{blank, ident::Ident, Parser};

#[derive(Debug, Clone)]
pub struct Graph {
    pub name: Ident,
    pub entry_node: Ident,
}

impl<'a> Parser<'a> for Graph {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("graph"),
                blank,
                Ident::parse,
                tag("("),
                Ident::parse,
                tag(")"),
            )),
            |(_, _, name, _, entry, _)| Graph {
                name,
                entry_node: entry,
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph() {
        let input = "graph Foo(Bar)";
        match super::Graph::parse(input) {
            Ok((remain, graph)) => {
                assert_eq!(remain, "");
                assert_eq!(graph.name.0, "Foo");
                assert_eq!(graph.entry_node.0, "Bar");
            }
            Err(e) => panic!("Error: {e:?}"),
        }
    }
}
