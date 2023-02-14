use std::sync::Arc;

use quote::{format_ident, quote, ToTokens};

use crate::symbol::Ident;

pub enum CodegenTy {
    String,
    Void,
    U8,
    Bool,
    Bytes,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Vec(Arc<CodegenTy>),
    Set(Arc<CodegenTy>),
    Map(Arc<CodegenTy>, Arc<CodegenTy>),
    ArcSwap(Arc<CodegenTy>),
    Adt(Adt),
}

pub struct Adt {
    pub segments: Arc<[Ident]>,
}

impl ToTokens for CodegenTy {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            CodegenTy::String => tokens.extend(quote! { ::std::string::String }),
            CodegenTy::Void => tokens.extend(quote! { () }),
            CodegenTy::U8 => tokens.extend(quote! { u8 }),
            CodegenTy::Bool => tokens.extend(quote! { bool }),
            CodegenTy::Bytes => tokens.extend(quote! { ::bytes::Bytes }),
            CodegenTy::I8 => tokens.extend(quote! { i8 }),
            CodegenTy::I16 => tokens.extend(quote! { i16 }),
            CodegenTy::I32 => tokens.extend(quote! { i32 }),
            CodegenTy::I64 => tokens.extend(quote! { i64 }),
            CodegenTy::F64 => tokens.extend(quote! { f64 }),
            CodegenTy::F32 => tokens.extend(quote! { f32 }),
            CodegenTy::Vec(ty) => {
                let ty = &**ty;
                tokens.extend(quote! { ::std::vec::Vec<#ty> });
            }
            CodegenTy::Set(ty) => {
                let ty = &**ty;
                tokens.extend(quote! { ::std::collections::HashSet<#ty> });
            }
            CodegenTy::Map(k, v) => {
                let k = &**k;
                let v = &**v;
                tokens.extend(quote! { ::std::collections::HashMap<#k, #v> });
            }
            CodegenTy::ArcSwap(ty) => {
                let ty = &**ty;
                tokens.extend(quote! { ::static_graph::ArcSwap<#ty> });
            }
            CodegenTy::Adt(adt) => {
                let adt: Vec<_> = adt
                    .segments
                    .iter()
                    .map(|ident| format_ident!("{}", ident))
                    .collect();

                tokens.extend(quote! { #(#adt)::* });
            }
        }
    }
}
