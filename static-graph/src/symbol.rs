use faststr::FastStr;
use heck::{ToSnakeCase, ToUpperCamelCase};
use quote::{format_ident, IdentFragment};
use std::fmt::Display;
use std::ops::Deref;

crate::newtype_index! {
    pub struct DefId { .. }
}

crate::newtype_index! {
    pub struct TagId { .. }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Symbol(pub FastStr);

impl std::borrow::Borrow<str> for Symbol {
    fn borrow(&self) -> &str {
        self
    }
}

impl Deref for Symbol {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for Symbol
where
    T: Into<FastStr>,
{
    fn from(t: T) -> Self {
        Symbol(t.into())
    }
}

impl IdentFragment for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Ident {
    pub sym: Symbol,
}

impl Ident {
    pub fn new(sym: Symbol) -> Self {
        Ident { sym }
    }
}

impl Deref for Ident {
    type Target = Symbol;

    fn deref(&self) -> &Self::Target {
        &self.sym
    }
}

impl IdentFragment for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        quote::IdentFragment::fmt(&self.sym, f)
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.sym, f)
    }
}

impl<T> From<T> for Ident
where
    T: Into<FastStr>,
{
    fn from(t: T) -> Self {
        Ident {
            sym: Symbol(t.into()),
        }
    }
}

pub trait IdentName {
    fn upper_camel_ident(&self) -> FastStr;

    fn snake_ident(&self) -> FastStr;

    fn as_syn_ident(&self) -> syn::Ident;
}

fn str2ident(s: &str) -> syn::Ident {
    format_ident!("{}", s)
}

impl IdentName for &str {
    fn upper_camel_ident(&self) -> FastStr {
        self.to_upper_camel_case().into()
    }

    fn snake_ident(&self) -> FastStr {
        self.to_snake_case().into()
    }

    fn as_syn_ident(&self) -> syn::Ident {
        str2ident(self)
    }
}

impl IdentName for FastStr {
    fn upper_camel_ident(&self) -> FastStr {
        (&**self).upper_camel_ident()
    }

    fn snake_ident(&self) -> FastStr {
        (&**self).snake_ident()
    }

    fn as_syn_ident(&self) -> syn::Ident {
        str2ident(self)
    }
}
