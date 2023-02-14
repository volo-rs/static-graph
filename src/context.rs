use std::sync::Arc;

use faststr::FastStr;
use fxhash::FxHashMap;

use crate::{
    resolver::rir::{Field, Graph, Node},
    symbol::{DefId, Ident, IdentName, TagId},
    tags::Tags,
};

pub struct Context {
    graphs: FxHashMap<DefId, Arc<Graph>>,
    nodes: FxHashMap<DefId, Arc<Node>>,
    fields: FxHashMap<DefId, Arc<Field>>,
    tags: FxHashMap<TagId, Arc<Tags>>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            graphs: Default::default(),
            nodes: Default::default(),
            fields: Default::default(),
            tags: Default::default(),
        }
    }

    pub fn set_graphs(&mut self, graphs: FxHashMap<DefId, Arc<Graph>>) {
        self.graphs = graphs;
    }

    pub fn set_nodes(&mut self, nodes: FxHashMap<DefId, Arc<Node>>) {
        self.nodes = nodes;
    }

    pub fn set_fields(&mut self, fields: FxHashMap<DefId, Arc<Field>>) {
        self.fields = fields;
    }

    pub fn set_tags(&mut self, tags: FxHashMap<TagId, Arc<Tags>>) {
        self.tags = tags;
    }

    pub fn graph(&self, graph_id: DefId) -> Option<Arc<Graph>> {
        self.graphs.get(&graph_id).cloned()
    }

    pub fn node(&self, node_id: DefId) -> Option<Arc<Node>> {
        self.nodes.get(&node_id).cloned()
    }

    pub fn field(&self, field_id: DefId) -> Option<Arc<Field>> {
        self.fields.get(&field_id).cloned()
    }

    pub fn tag(&self, tag_id: TagId) -> Option<Arc<Tags>> {
        self.tags.get(&tag_id).cloned()
    }

    pub fn snake_name(&self, ident: &Ident) -> FastStr {
        (&***ident).snake_ident()
    }

    pub fn upper_camel_name(&self, ident: &Ident) -> FastStr {
        (&***ident).upper_camel_ident()
    }
}
