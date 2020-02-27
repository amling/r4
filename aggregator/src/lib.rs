#[macro_use]
extern crate lazy_static;
extern crate misc;
extern crate rand;
extern crate record;
#[macro_use]
extern crate registry;
extern crate registry_args;
#[macro_use]
extern crate registry_args_derive;
extern crate validates;

use record::Record;
use registry::Registrant;
use registry_args::RegistryArgs;
use std::sync::Arc;

pub type BoxedAggregator = Box<dyn AggregatorInbox>;

registry! {
    BoxedAggregator,
    array,
    average,
    concat,
    count,
    count_by,
    distinct_array,
    distinct_concat,
    distinct_count,
    first,
    first_record,
    hash,
    last,
    last_record,
    lexical_max,
    lexical_min,
    lexical_percentile,
    linear_regression,
    max,
    min,
    percentile,
    record_for_lexical_max,
    record_for_lexical_min,
    record_for_lexical_percentile,
    record_for_max,
    record_for_min,
    record_for_percentile,
    records,
    sample,
    standard_deviation,
    sum,
}

trait AggregatorBe {
    type Args: RegistryArgs;
    type State: Clone + Default + Send + Sync;

    fn names() -> Vec<&'static str>;
    fn help_msg() -> &'static str;
    fn add(state: &mut Self::State, a: &Self::Args, r: Record);
    fn finish(state: Self::State, a: &Self::Args) -> Record;
}

pub trait AggregatorInbox: Send + Sync {
    fn add(&mut self, r: Record);
    fn finish(self: Box<Self>) -> Record;
    fn box_clone(&self) -> BoxedAggregator;
}

impl Clone for BoxedAggregator {
    fn clone(&self) -> BoxedAggregator {
        return self.box_clone();
    }
}

struct AggregatorInboxImpl<B: AggregatorBe> {
    a: Arc<B::Args>,
    s: B::State,
}

impl<B: AggregatorBe + 'static> AggregatorInbox for AggregatorInboxImpl<B> {
    fn add(&mut self, r: Record) {
        B::add(&mut self.s, &self.a, r);
    }

    fn finish(self: Box<Self>) -> Record {
        let a = self.a.clone();
        return B::finish(self.s, &a);
    }

    fn box_clone(&self) -> BoxedAggregator {
        return Box::new(AggregatorInboxImpl::<B> {
            a: self.a.clone(),
            s: self.s.clone(),
        });
    }
}

struct AggregatorRegistrant<B: AggregatorBe> {
    _b: std::marker::PhantomData<B>,
}

impl<B: AggregatorBe + 'static> Registrant<BoxedAggregator> for AggregatorRegistrant<B> {
    type Args = B::Args;

    fn names() -> Vec<&'static str> {
        return B::names();
    }

    fn help_msg() -> &'static str {
        return B::help_msg();
    }

    fn init(a: B::Args) -> BoxedAggregator {
        return Box::new(AggregatorInboxImpl::<B>{
            a: Arc::new(a),
            s: B::State::default(),
        });
    }
}
