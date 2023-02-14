#[cfg(test)]
mod tests {
    use crate::parse::Parser;

    #[test]
    fn test_parse_document() {
        let input = r#"
        node E -> (X, Y) {
            e: string,
            #[default = "Cache::new"]
            cache: cache::Cache,
        }
        node X -> O {
            x: string,
        }
        node Y -> O {
            y: string,
        }
        node O {

        }
        graph G(E)
        "#;
        let res = super::Document::parse(input);
        eprintln!("{res:?}");
    }

    #[test]
    fn test_parse_field() {
        let input = r#"#[default = "String::new"]
        a: string,
        "#;
        let res = super::Field::parse(input);
        eprintln!("{res:?}");
    }

    #[test]
    fn test_parse_node() {
        let input = r#"node E {
            #[default = "String::new", a = "b"]
            a: string,
            b: i32,
        }"#;
        let res = super::Node::parse(input);
        eprintln!("{res:?}");
    }
}
