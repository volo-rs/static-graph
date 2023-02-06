pub mod rir;

use std::{str::FromStr, sync::Arc};

use fxhash::FxHashMap;

use crate::{
    index::Idx,
    parser::document::Document,
    symbol::{DefId, Ident, Symbol, TagId},
    tags::Annotation,
    tags::{Construct, Tags},
};

use self::rir::{Field, Graph, Node, Path, Type};

pub struct Resolver {
    graphs: FxHashMap<DefId, Arc<Graph>>,
    nodes: FxHashMap<DefId, Arc<Node>>,
    fields: FxHashMap<DefId, Arc<Field>>,
    tags: FxHashMap<TagId, Arc<Tags>>,
    did_counter: DefId,
    tid_counter: TagId,
    symbol_table: FxHashMap<Symbol, DefId>,
}

impl Default for Resolver {
    fn default() -> Self {
        Self {
            graphs: Default::default(),
            nodes: Default::default(),
            fields: Default::default(),
            tags: Default::default(),
            did_counter: DefId::from_usize(0),
            tid_counter: TagId::from_usize(0),
            symbol_table: Default::default(),
        }
    }
}

pub struct ResolveResult {
    pub graphs: FxHashMap<DefId, Arc<Graph>>,
    pub nodes: FxHashMap<DefId, Arc<Node>>,
    pub fields: FxHashMap<DefId, Arc<Field>>,
    pub tags: FxHashMap<TagId, Arc<Tags>>,
    pub entrys: Vec<DefId>,
}

impl Resolver {
    pub fn resolve_document(mut self, document: Document) -> ResolveResult {
        let _ = document
            .nodes
            .iter()
            .map(|node| self.lower_node(node))
            .collect::<Vec<_>>();
        let entrys: Vec<_> = document
            .graphs
            .iter()
            .map(|graph| self.lower_graph(graph))
            .collect();
        ResolveResult {
            graphs: self.graphs,
            nodes: self.nodes,
            fields: self.fields,
            tags: self.tags,
            entrys,
        }
    }

    fn lower_node(&mut self, n: &crate::parser::node::Node) -> Arc<Node> {
        let name = self.lower_ident(&n.name);
        let def_id = self.get_did(&name);
        let to_nodes = n
            .to_nodes
            .iter()
            .map(|n| {
                let ident = self.lower_ident(n);
                self.get_did(&ident)
            })
            .collect();
        let fields = n
            .fields
            .iter()
            .map(|field| self.lower_field(field))
            .collect();

        let node = Arc::from(Node {
            name,
            to_nodes,
            fields,
        });

        self.nodes.insert(def_id, node.clone());

        node
    }

    fn get_did(&mut self, name: &Ident) -> DefId {
        *self
            .symbol_table
            .entry(name.clone().sym)
            .or_insert(self.did_counter.inc_one())
    }

    fn lower_graph(&mut self, g: &crate::parser::graph::Graph) -> DefId {
        let name = self.lower_ident(&g.name);
        let def_id = self.get_did(&name);

        let entry_node_name = self.lower_ident(&g.entry_node);
        let entry_node_def_id = self.get_did(&entry_node_name);

        let graph = Arc::from(Graph {
            name,
            entry_node: entry_node_def_id,
        });

        self.graphs.insert(def_id, graph);

        def_id
    }

    fn lower_field(&mut self, f: &crate::parser::field::Field) -> Arc<Field> {
        let tag_id = self.tid_counter.inc_one();
        let tags = self.extract_tags(&f.annotations);
        self.tags.insert(tag_id, tags.into());

        let name = self.lower_ident(&f.name);
        let def_id = self.get_did(&name);
        let ty = self.lower_type(&f.ty);

        let field = Arc::from(Field { name, ty, tag_id });

        self.fields.insert(def_id, field.clone());

        field
    }

    fn extract_tags(&mut self, annotations: &crate::parser::annotations::Annotations) -> Tags {
        let mut tags = Tags::default();
        macro_rules! with_tags {
            ($annotation: tt -> $($key: ty)|+) => {
                match $annotation.key.as_str()  {
                    $(<$key>::KEY => {
                        tags.insert(<$key>::from_str(&$annotation.value).unwrap());
                    }),+
                    _ => {},
                }
            };
        }

        annotations
            .iter()
            .for_each(|annotation| with_tags!(annotation -> Construct));

        tags
    }

    fn lower_type(&mut self, ty: &crate::parser::ty::Type) -> Type {
        match ty {
            crate::parser::ty::Type::String => Type::String,
            crate::parser::ty::Type::Void => Type::Void,
            crate::parser::ty::Type::Byte => Type::U8,
            crate::parser::ty::Type::Bool => Type::Bool,
            crate::parser::ty::Type::Binary => Type::Bytes,
            crate::parser::ty::Type::I8 => Type::I8,
            crate::parser::ty::Type::I16 => Type::I16,
            crate::parser::ty::Type::I32 => Type::I32,
            crate::parser::ty::Type::I64 => Type::I64,
            crate::parser::ty::Type::Double => Type::F64,
            crate::parser::ty::Type::List { value } => Type::Vec(Arc::from(self.lower_type(value))),
            crate::parser::ty::Type::Set { value } => Type::Set(Arc::from(self.lower_type(value))),
            crate::parser::ty::Type::Map { key, value } => Type::Map(
                Arc::from(self.lower_type(key)),
                Arc::from(self.lower_type(value)),
            ),
            crate::parser::ty::Type::Path(path) => Type::Path(self.lower_path(path)),
        }
    }

    fn lower_ident(&mut self, ident: &crate::parser::ident::Ident) -> Ident {
        Ident::from(ident.0.clone())
    }

    fn lower_path(&mut self, path: &crate::parser::path::Path) -> Path {
        Path {
            segments: Arc::from_iter(path.segments.iter().map(|i| self.lower_ident(i))),
        }
    }
}
