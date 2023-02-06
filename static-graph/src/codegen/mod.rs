pub mod ty;

use std::sync::Arc;
use std::{collections::VecDeque, ops::Deref};

use faststr::FastStr;
use fxhash::{FxHashMap, FxHashSet};
use proc_macro2::TokenStream;
use quote::format_ident;

use crate::{
    context::Context,
    resolver::rir::{Graph, Node, Type},
    symbol::{DefId, IdentName},
    tags::Construct,
};

pub struct Codegen {
    cx: Context,
    nesteds: FxHashMap<DefId, FastStr>,
    in_degrees: FxHashMap<DefId, u32>,
    froms: FxHashMap<DefId, Vec<DefId>>,
    tos: FxHashMap<DefId, Vec<DefId>>,
    visited: FxHashSet<DefId>,
}

impl Deref for Codegen {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.cx
    }
}

impl Codegen {
    pub fn new(cx: Context) -> Self {
        Self {
            cx,
            nesteds: FxHashMap::default(),
            in_degrees: FxHashMap::default(),
            froms: FxHashMap::default(),
            tos: FxHashMap::default(),
            visited: FxHashSet::default(),
        }
    }

    pub fn write_document(&mut self, def_ids: Vec<DefId>) -> TokenStream {
        let mut stream = TokenStream::new();
        self.write_trait(&mut stream);
        for def_id in def_ids {
            self.write_graph(def_id, &mut stream);
        }
        stream
    }

    pub fn write_graph(&mut self, def_id: DefId, stream: &mut TokenStream) {
        let graph = self.graph(def_id).unwrap();
        let graph_name = self.upper_camel_name(&graph.name).as_syn_ident();

        let entry_node = self.node(graph.entry_node).unwrap();
        let entry_node_name = self.snake_name(&entry_node.name).as_syn_ident();
        let entry_node_ty = self.upper_camel_name(&entry_node.name).as_syn_ident();

        stream.extend(quote::quote! {
            pub struct #graph_name {
                #entry_node_name: ::std::sync::Arc<#entry_node_ty>,
            }
            impl #graph_name {
                pub fn new() -> Self {
                    Self {
                        #entry_node_name: ::std::sync::Arc::new(#entry_node_ty::new()),
                    }
                }
            }
        });

        self.write_node(graph.entry_node, &entry_node, stream, &"self".into());

        self.write_run(graph, stream);
    }

    pub fn write_node(
        &mut self,
        def_id: DefId,
        node: &Arc<Node>,
        stream: &mut TokenStream,
        nested: &FastStr,
    ) {
        if self.visited.contains(&def_id) {
            return;
        }
        self.visited.insert(def_id);

        let name = self.upper_camel_name(&node.name).as_syn_ident();
        let mut nodes = TokenStream::new();
        let mut nodes_impl = TokenStream::new();
        for did in &node.to_nodes {
            let node = self.node(*did).unwrap();
            let name = self.snake_name(&node.name).as_syn_ident();
            let ty = self.upper_camel_name(&node.name).as_syn_ident();
            self.in_degrees
                .entry(*did)
                .and_modify(|e| *e += 1)
                .or_insert(1);
            self.tos.entry(def_id).or_default().push(*did);
            self.froms.entry(*did).or_default().push(def_id);
            nodes.extend(quote::quote! {
                pub #name: ::std::sync::Arc<#ty>,
            });
            nodes_impl.extend(quote::quote! {
                #name: ::std::sync::Arc::new(#ty::new()),
            });
        }

        let mut fields = TokenStream::new();
        let mut fields_impl = TokenStream::new();
        for f in &node.fields {
            let name = self.snake_name(&f.name).as_syn_ident();
            let ty = f.ty.to_codegen_ty();
            fields.extend(quote::quote! {
                pub #name: #ty,
            });
            match &f.ty {
                Type::Path(_) => {
                    let tags = self.tag(f.tag_id).unwrap();
                    let c = tags.get::<Construct>().unwrap();

                    let ident: Vec<_> = c.0.split("::").map(|s| format_ident!("{}", s)).collect();

                    fields_impl.extend(quote::quote! {
                        #name: #(#ident)::*(),
                    });
                }
                _ => {
                    fields_impl.extend(quote::quote! {
                        #name: Default::default(),
                    });
                }
            }
        }

        stream.extend(quote::quote! {
            pub struct #name {
                #nodes
                #fields
            }
            impl #name {
                pub fn new() -> Self {
                    Self {
                        #nodes_impl
                        #fields_impl
                    }
                }
            }
        });

        let nested: FastStr = format!("{}.{}", nested, self.snake_name(&node.name)).into();
        self.nesteds.insert(def_id, nested.clone());
        for did in &node.to_nodes {
            self.write_node(*did, &self.node(*did).unwrap(), stream, &nested);
        }
    }

    #[inline]
    fn write_trait(&mut self, stream: &mut TokenStream) {
        stream.extend(quote::quote! {
            #[static_graph::async_trait]
            pub trait Runnable<Req, PrevResp>
            where
                Req: Clone,
            {
                type Resp;
                async fn run(&self, req: Req, prev_resp: PrevResp) -> Self::Resp;
            }
        });
    }

    fn write_run(&mut self, graph: Arc<Graph>, stream: &mut TokenStream) {
        let name = self.upper_camel_name(&graph.name).as_syn_ident();
        let mut visited = FxHashSet::default();
        let mut queue = VecDeque::new();

        assert!(self.in_degrees.get(&graph.entry_node).is_none());

        queue.push_back(graph.entry_node);
        let mut bounds = TokenStream::new();
        let mut bodys = TokenStream::new();
        let mut generics = Vec::new();
        let mut out_resp = None;
        while !queue.is_empty() {
            let sz = queue.len();
            for _ in 0..sz {
                let did = queue.pop_front().unwrap();
                let node = self.node(did).unwrap();
                let name = self.snake_name(&node.name).as_syn_ident();
                let upper_name = self.upper_camel_name(&node.name).as_syn_ident();

                let mut upper_prev_resps = Vec::new();
                let mut prev_resps = Vec::new();
                let mut prev_resps_clones = Vec::new();
                if let Some(from_dids) = self.froms.get(&did) {
                    let mut resps = Vec::with_capacity(from_dids.len());
                    let mut handles = Vec::with_capacity(from_dids.len());
                    let mut matches = Vec::with_capacity(from_dids.len());

                    for from_did in from_dids {
                        let node = self.node(*from_did).unwrap();

                        let f_name = self.snake_name(&node.name).as_syn_ident();
                        let upper_f_name = self.upper_camel_name(&node.name).as_syn_ident();
                        let upper_prev_resp = format_ident!("{}Resp", upper_f_name);

                        let prev_resp = format_ident!("{}_{}_resp", name, f_name);
                        let resp = format_ident!("{}_resp", f_name);

                        if !visited.contains(from_did) {
                            resps.push(resp.clone());
                            handles.push(format_ident!("{}_handle", f_name));
                            matches.push(quote::quote! {
                                Ok(#resp)
                            });
                            visited.insert(from_did);
                        }

                        upper_prev_resps.push(upper_prev_resp);
                        prev_resps.push(prev_resp.clone());
                        prev_resps_clones.push(quote::quote! {
                            let #prev_resp = #resp.clone();
                        });
                    }

                    if !resps.is_empty() {
                        bodys.extend(quote::quote! {
                            let (#(#resps),*) = match static_graph::join!(#(#handles),*) {
                                (#(#matches,)*) => (#(#resps),*),
                                _ => panic!("Error"),
                            };
                        });
                    }
                };

                let upper_resp = format_ident!("{}Resp", upper_name);
                generics.push(upper_resp.clone());
                bounds.extend(quote::quote! {
                    #upper_name: Runnable<Req, (#(#upper_prev_resps),*), Resp = #upper_resp>,
                    #upper_resp: Clone + Send + Sync + 'static,
                });

                let req = format_ident!("{}_req", name);
                let handle = format_ident!("{}_handle", name);
                let node: Vec<_> = self
                    .nesteds
                    .get(&did)
                    .unwrap()
                    .split('.')
                    .map(|s| format_ident!("{}", s))
                    .collect();

                if let Some(to_dids) = self.tos.get(&did) {
                    for to_did in to_dids {
                        if let Some(in_degree) = self.in_degrees.get_mut(to_did) {
                            *in_degree -= 1;
                            if *in_degree == 0 {
                                self.in_degrees.remove(to_did);
                                queue.push_back(*to_did);
                            }
                        }
                    }
                    bodys.extend(quote::quote! {
                        let #req = req.clone();
                        #(#prev_resps_clones)*
                        let #name = #(#node.)*clone();
                        let #handle = static_graph::spawn(async move {
                            #name.run(#req, (#(#prev_resps),*)).await
                        });
                    });
                } else {
                    assert!(out_resp.is_none());

                    out_resp.replace(upper_resp);
                    bodys.extend(quote::quote! {
                        #(#prev_resps_clones)*
                        #(#node).*.run(req, (#(#prev_resps),*)).await
                    });
                }
            }
        }

        assert!(self.in_degrees.is_empty());

        let out_resp = out_resp.unwrap();
        stream.extend(quote::quote! {
            impl #name {
                pub async fn run<Req, #(#generics),*>(&self, req: Req) -> #out_resp
                where
                    Req: Clone + Send + Sync + 'static,
                    #bounds
                {
                    #bodys
                }
            }
        });
    }
}
