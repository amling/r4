use record::Record;
use record::RecordTrait;
use registry::args::OneKeyRegistryArgs;
use std::collections::HashMap;
use std::hash::Hash;
use super::AggregatorBe;
use super::AggregatorRegistrant;

#[derive(Clone)]
pub struct DistinctSet<T> {
    v: Vec<T>,
    m: HashMap<T, ()>,
}

impl<T: Eq + Hash> Default for DistinctSet<T> {
    fn default() -> Self {
        return DistinctSet {
            v: Vec::new(),
            m: HashMap::new(),
        };
    }
}

impl<T: Clone + Eq + Hash> DistinctSet<T> {
    pub fn add(&mut self, t: T) {
        if self.m.insert(t.clone(), ()).is_none() {
            self.v.push(t);
        }
    }
}

impl<T> IntoIterator for DistinctSet<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> std::vec::IntoIter<T> {
        return self.v.into_iter();
    }
}

pub(crate) type Impl = AggregatorRegistrant<ImplBe>;

pub(crate) struct ImplBe;

impl AggregatorBe for ImplBe {
    type Args = OneKeyRegistryArgs;
    type State = DistinctSet<Record>;

    fn names() -> Vec<&'static str> {
        return vec!["darray", "darr"];
    }

    fn help_msg() -> &'static str {
        return "collect distinct values into an array";
    }

    fn add(state: &mut DistinctSet<Record>, a: &OneKeyRegistryArgs, r: Record) {
        state.add(r.get_path(&a.key));
    }

    fn finish(state: DistinctSet<Record>, _a: &OneKeyRegistryArgs) -> Record {
        return Record::from_vec(state.into_iter().collect());
    }
}
