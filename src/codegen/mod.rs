pub mod ty;

use std::sync::Arc;
use std::{collections::VecDeque, ops::Deref};

use faststr::FastStr;
use fxhash::{FxHashMap, FxHashSet};
use proc_macro2::TokenStream;
use quote::format_ident;

use crate::tags::Editable;
use crate::{
    context::Context,
    resolver::rir::{Graph, Node},
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
                pub #entry_node_name: ::std::sync::Arc<#entry_node_ty>,
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

            let tags = self.tag(f.tag_id).unwrap();
            if let Some(c) = tags.get::<Construct>() {
                let ident: Vec<_> = c.0.split("::").map(|s| format_ident!("{}", s)).collect();
                if let Some(Editable(true)) = tags.get::<Editable>() {
                    fields_impl.extend(quote::quote! {
                        #name: ::static_graph::ArcSwap::from_pointee(#(#ident)::*()),
                    });
                } else {
                    fields_impl.extend(quote::quote! {
                        #name: #(#ident)::*(),
                    });
                }
            } else {
                fields_impl.extend(quote::quote! {
                    #name: ::std::default::Default::default(),
                });
            };
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
            pub trait Runnable<Req, PrevResp> {
                type Resp;
                type Error;
                fn run(&self, req: Req, prev_resp: PrevResp) -> impl std::future::Future<Output = ::std::result::Result<Self::Resp, Self::Error>> + Send;
            }
        });
    }

    fn write_run(&mut self, graph: Arc<Graph>, stream: &mut TokenStream) {
        let name = self.upper_camel_name(&graph.name).as_syn_ident();
        let mut queue = VecDeque::new();

        assert!(!self.in_degrees.contains_key(&graph.entry_node));

        queue.push_back(graph.entry_node);
        let mut bounds = TokenStream::new();
        let mut bodys = TokenStream::new();
        let mut generics = Vec::new();
        let mut out_resp = None;
        while !queue.is_empty() {
            let sz = queue.len();
            for _ in 0..sz {
                let mut channels = TokenStream::new();

                let did = queue.pop_front().unwrap();
                let node = self.node(did).unwrap();
                let name = self.snake_name(&node.name).as_syn_ident();
                let upper_name = self.upper_camel_name(&node.name).as_syn_ident();

                let mut upper_prev_resps = Vec::new();
                let mut resps = Vec::new();
                if let Some(from_dids) = self.froms.get(&did) {
                    let mut rxs = Vec::with_capacity(from_dids.len());
                    let mut matches = Vec::with_capacity(from_dids.len());

                    for from_did in from_dids {
                        let node = self.node(*from_did).unwrap();

                        let f_name = self.snake_name(&node.name).as_syn_ident();
                        let upper_f_name = self.upper_camel_name(&node.name).as_syn_ident();
                        let upper_prev_resp = format_ident!("{}Resp", upper_f_name);

                        let resp = format_ident!("{}_resp", f_name);

                        resps.push(resp.clone());
                        rxs.push(format_ident!("{}_rx_{}", name, f_name));
                        matches.push(quote::quote! {
                            Ok(Ok(#resp))
                        });

                        upper_prev_resps.push(upper_prev_resp);
                    }

                    if !resps.is_empty() {
                        channels.extend(quote::quote! {
                            let (#(#resps),*) = match static_graph::join!(#(#rxs.recv()),*) {
                                (#(#matches,)*) => (#(#resps),*),
                                _ => panic!("Error"),
                            };
                        });
                    }
                };

                let upper_resp = format_ident!("{}Resp", upper_name);
                generics.push(upper_resp.clone());
                bounds.extend(quote::quote! {
                    #upper_name: Runnable<Req, (#(#upper_prev_resps),*), Resp = #upper_resp, Error = Error>,
                    #upper_resp: Clone + Send + Sync + 'static,
                });

                let req = format_ident!("{}_req", name);
                let tx = format_ident!("{}_tx", name);
                let node: Vec<_> = self
                    .nesteds
                    .get(&did)
                    .unwrap()
                    .split('.')
                    .map(|s| format_ident!("{}", s))
                    .collect();

                if let Some(to_dids) = self.tos.get(&did) {
                    let mut rxs = Vec::with_capacity(to_dids.len());
                    let len = to_dids.len() + 1;
                    for to_did in to_dids {
                        if let Some(in_degree) = self.in_degrees.get_mut(to_did) {
                            *in_degree -= 1;
                            if *in_degree == 0 {
                                self.in_degrees.remove(to_did);
                                queue.push_back(*to_did);
                            }
                        }
                        let node = self.node(*to_did).unwrap();
                        let to_name = self.snake_name(&node.name).as_syn_ident();
                        rxs.push(format_ident!("{}_rx_{}", to_name, name));
                    }
                    bodys.extend(quote::quote! {
                        let #req = req.clone();
                        let #name = #(#node.)*clone();
                        let (#tx, _) = static_graph::sync::broadcast::channel(#len);
                        #(let mut #rxs = #tx.subscribe();)*
                        static_graph::spawn(async move {
                            #channels
                            let resp = #name.run(#req, (#(#resps),*)).await;
                            #tx.send(resp).ok();
                        });
                    });
                } else {
                    assert!(out_resp.is_none());

                    out_resp.replace(upper_resp);
                    bodys.extend(quote::quote! {
                        #channels
                        #(#node).*.run(req, (#(#resps),*)).await
                    });
                }
            }
        }

        assert!(self.in_degrees.is_empty());

        let out_resp = out_resp.unwrap();
        stream.extend(quote::quote! {
            impl #name {
                pub async fn run<Req, #(#generics),*, Error>(&self, req: Req) -> ::std::result::Result<#out_resp, Error>
                where
                    Req: Clone + Send + Sync + 'static,
                    Error: Clone + Send + Sync + 'static,
                    #bounds
                {
                    #bodys
                }
            }
        });
    }
}
