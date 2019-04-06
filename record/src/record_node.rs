use misc::Either;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec::Vec;
use super::JsonPrimitive;
use super::PathStep;
use super::RPathStep;

#[derive(Clone)]
#[derive(Debug)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub enum RecordNode<T> {
    Primitive(JsonPrimitive),
    Array(Vec<T>),
    Hash(BTreeMap<Arc<str>, T>),
}

impl<T, F> From<F> for RecordNode<T> where JsonPrimitive: From<F> {
    fn from(f: F) -> Self {
        return RecordNode::Primitive(JsonPrimitive::from(f));
    }
}

impl<T> RecordNode<T> {
    pub fn map<S, F: Fn(T) -> S>(self, f: F) -> RecordNode<S> {
        return match self {
            RecordNode::Primitive(p) => RecordNode::Primitive(p),
            RecordNode::Array(arr) => RecordNode::Array(arr.into_iter().map(f).collect()),
            RecordNode::Hash(hash) => RecordNode::Hash(hash.into_iter().map(|(k, v)| (k, f(v))).collect()),
        };
    }

    pub fn maybe_primitive(&self) -> Option<JsonPrimitive> {
        return match self {
            RecordNode::Primitive(p) => Some(p.clone()),
            _ => None,
        };
    }
}

pub trait RecordTrait: Sized {
    fn new(n: RecordNode<Self>) -> Self;

    fn null() -> Self {
        return Self::new(RecordNode::from(JsonPrimitive::Null()));
    }

    fn empty_hash() -> Self {
        return Self::from_hash(BTreeMap::new());
    }

    fn from_vec(arr: Vec<Self>) -> Self {
        return Self::new(RecordNode::Array(arr));
    }

    fn from_hash(hash: BTreeMap<Arc<str>, Self>) -> Self {
        return Self::new(RecordNode::Hash(hash));
    }

    fn maybe_primitive(&self) -> Option<JsonPrimitive>;

    fn coerce_num(&self) -> Either<i64, f64> {
        match self.maybe_primitive() {
            Some(JsonPrimitive::Null()) => {
                return Either::Left(0);
            }
            Some(JsonPrimitive::NumberI64(n)) => {
                return Either::Left(n);
            }
            Some(JsonPrimitive::NumberF64(ref n)) => {
                return Either::Right(n.0);
            }
            Some(JsonPrimitive::String(s)) => {
                if let Ok(n) = s.parse() {
                    return Either::Left(n);
                }
                if let Ok(n) = s.parse() {
                    return Either::Right(n);
                }
                panic!("coerce_num() on unparseable string {}", s);
            }
            _ => {
                panic!("coerce_num() on something incoercible");
            }
        }
    }

    fn coerce_string(&self) -> Arc<str> {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::Null()) => Arc::from(""),
            Some(JsonPrimitive::Bool(b)) => Arc::from(b.to_string()),
            Some(JsonPrimitive::NumberF64(ref f)) => Arc::from(f.0.to_string()),
            Some(JsonPrimitive::NumberI64(i)) => Arc::from(i.to_string()),
            Some(JsonPrimitive::String(ref s)) => s.clone(),
            _ => panic!("coerce_string() on something incoercible"),
        };
    }

    fn coerce_bool(&self) -> bool {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::Null()) => false,
            Some(JsonPrimitive::Bool(b)) => b,
            Some(JsonPrimitive::NumberF64(ref f)) => f.0 != 0.0,
            Some(JsonPrimitive::NumberI64(i)) => i != 0,
            Some(JsonPrimitive::String(ref s)) => !s.is_empty(),
            None => true,
        };
    }

    fn coerce_f64(&self) -> f64 {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::NumberF64(ref f)) => f.0,
            Some(JsonPrimitive::NumberI64(i)) => i as f64,
            Some(JsonPrimitive::String(ref s)) => s.parse().unwrap(),
            _ => panic!("coerce_f64() on something incoercible"),
        };
    }

    fn expect_string(&self) -> Arc<str> {
        return match self.maybe_primitive() {
            Some(JsonPrimitive::String(ref s)) => s.clone(),
            _ => panic!("expect_string() on non-string"),
        };
    }
}

impl<T: RecordTrait> RecordNode<T> {
    pub fn get_rstep(&self, step: &PathStep) -> Option<&T> {
        match step.as_r() {
            RPathStep::Hash(s) => {
                if let RecordNode::Hash(hash) = self {
                    return hash.get(s);
                }
                panic!("hash step on non-hash");
            }
            RPathStep::Array(n) => {
                if let RecordNode::Array(arr) = self {
                    return arr.get(n);
                }
                panic!("array step on non-array");
            }
        }
    }

    pub fn get_rstep_mut(&mut self, step: &PathStep) -> Option<&mut T> {
        match step.as_r() {
            RPathStep::Hash(s) => {
                if let RecordNode::Hash(hash) = self {
                    return hash.get_mut(s);
                }
                panic!("hash step on non-hash");
            }
            RPathStep::Array(n) => {
                if let RecordNode::Array(arr) = self {
                    return arr.get_mut(n);
                }
                panic!("array step on non-array");
            }
        }
    }

    pub fn get_rstep_fill(&mut self, step: &PathStep) -> &mut T {
        // We don't as_r() because we want to avoid making our own Arc in the
        // OwnHash case (preferring to take another reference to the existing
        // one).
        match step {
            PathStep::RefHash(s) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Hash(BTreeMap::new());
                }
                if let RecordNode::Hash(hash) = self {
                    return hash.entry(Arc::from(*s)).or_insert_with(T::null);
                }
                panic!("hash step on non-hash");
            }
            PathStep::OwnHash(s) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Hash(BTreeMap::new());
                }
                if let RecordNode::Hash(hash) = self {
                    return hash.entry(s.clone()).or_insert_with(T::null);
                }
                panic!("hash step on non-hash");
            }
            PathStep::Array(n) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Array(Vec::new());
                }
                if let RecordNode::Array(arr) = self {
                    while *n >= arr.len() {
                        arr.push(T::null());
                    }
                    return &mut arr[*n];
                }
                panic!("array step on non-array");
            }
        }
    }

    pub fn del_rpart(&mut self, step: &PathStep) -> T {
        match step.as_r() {
            RPathStep::Hash(s) => {
                if let RecordNode::Primitive(JsonPrimitive::Null()) = self {
                    *self = RecordNode::Hash(BTreeMap::new());
                }
                if let RecordNode::Hash(hash) = self {
                    return hash.remove(s).unwrap_or_else(T::null);
                }
                panic!("delete hash step on non-hash");
            }
            RPathStep::Array(_n) => {
                panic!("delete array step");
            }
        }
    }
}
