use std::sync::Arc;

use crate::{
    codegen::ty::{Adt, CodegenTy},
    symbol::{DefId, Ident, TagId},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Graph {
    pub name: Ident,
    pub entry_node: DefId,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Node {
    pub name: Ident,
    pub to_nodes: Vec<DefId>,
    pub fields: Vec<Arc<Field>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub tag_id: TagId,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Path {
    pub segments: Arc<[Ident]>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    String,
    Void,
    U8,
    Bool,
    Bytes,
    I8,
    I16,
    I32,
    I64,
    F64,
    Vec(Arc<Type>),
    Set(Arc<Type>),
    Map(Arc<Type>, Arc<Type>),
    ArcSwap(Arc<Type>),
    Path(Path),
}

impl Type {
    pub fn to_codegen_ty(&self) -> CodegenTy {
        match self {
            Type::String => CodegenTy::String,
            Type::Void => CodegenTy::Void,
            Type::U8 => CodegenTy::U8,
            Type::Bool => CodegenTy::Bool,
            Type::Bytes => CodegenTy::Bytes,
            Type::I8 => CodegenTy::I8,
            Type::I16 => CodegenTy::I16,
            Type::I32 => CodegenTy::I32,
            Type::I64 => CodegenTy::I64,
            Type::F64 => CodegenTy::F64,
            Type::Vec(ty) => CodegenTy::Vec(Arc::from(ty.to_codegen_ty())),
            Type::Set(ty) => CodegenTy::Set(Arc::from(ty.to_codegen_ty())),
            Type::Map(k, v) => {
                CodegenTy::Map(Arc::from(k.to_codegen_ty()), Arc::from(v.to_codegen_ty()))
            }
            Type::ArcSwap(ty) => CodegenTy::ArcSwap(Arc::from(ty.to_codegen_ty())),
            Type::Path(p) => CodegenTy::Adt(Adt {
                segments: p.segments.clone(),
            }),
        }
    }
}
