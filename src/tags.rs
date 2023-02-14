use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use faststr::FastStr;

#[derive(Default, Debug)]
pub struct TypeMap(HashMap<TypeId, Box<dyn Any + Sync + Send>>);

impl TypeMap {
    pub fn insert<T: 'static + Sync + Send>(&mut self, v: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(v));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .map(|v| v.downcast_ref().unwrap())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn contains<T: 'static>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<T>())
    }

    pub fn remove<T: 'static>(&mut self) {
        self.0.remove(&TypeId::of::<T>());
    }
}

#[derive(Default, Debug)]
pub struct Tags(TypeMap);

impl Deref for Tags {
    type Target = TypeMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Tags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub struct Construct(pub FastStr);

pub trait Annotation: FromStr {
    const KEY: &'static str;
}

impl FromStr for Construct {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(FastStr::new(s)))
    }
}

impl Annotation for Construct {
    const KEY: &'static str = "default";
}

#[derive(Clone)]
pub struct Editable(pub bool);

impl FromStr for Editable {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s == "true"))
    }
}

impl Annotation for Editable {
    const KEY: &'static str = "editable";
}
