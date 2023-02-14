use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::many0,
    sequence::tuple,
    IResult,
};

use super::{blank, graph::Graph, node::Node, Parser};

#[derive(Debug, Clone)]
pub struct Document {
    pub graphs: Vec<Graph>,
    pub nodes: Vec<Node>,
}

impl<'a> Parser<'a> for Document {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        enum NodeOrGraph {
            Node(Node),
            Graph(Graph),
        }
        map(
            many0(map(
                tuple((
                    opt(blank),
                    alt((
                        map(Node::parse, NodeOrGraph::Node),
                        map(Graph::parse, NodeOrGraph::Graph),
                    )),
                    opt(blank),
                )),
                |(_, nog, _)| nog,
            )),
            |nogs| {
                let mut graphs = Vec::new();
                let mut nodes = Vec::new();
                for nog in nogs.into_iter() {
                    match nog {
                        NodeOrGraph::Node(node) => nodes.push(node),
                        NodeOrGraph::Graph(graph) => graphs.push(graph),
                    }
                }
                Document { graphs, nodes }
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document() {
        let input = r#"
        graph Foo(Bar)

        node Bar -> (Baz, Qux) {
            #[default = "B::new"]
            b: B,
        }

        node Baz -> Out {
            #[default = "C::new"]
            c: C,
        }

        node Qux -> Out {
            #[default = "D::new"]
            d: D,
        }

        node Out {

        }
        "#;

        match super::Document::parse(input) {
            Ok((remain, doc)) => {
                assert_eq!(remain, "");
                assert_eq!(doc.graphs.len(), 1);
                assert_eq!(doc.nodes.len(), 4);
            }
            Err(e) => panic!("Error: {e:?}"),
        }
    }
}
