use opts::help::ToOptionsHelp;
use opts::parser::OptionsPile;
use opts::vals::UnvalidatedOption;
use record::Record;
use sorts::BoxedSort;
use sorts::bucket::SortBucket;
use sorts::bucket::VecDequeSortBucket;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
#[derive(Validates)]
pub struct SortOptions(UnvalidatedOption<Vec<BoxedSort>>);

impl SortOptions {
    pub fn options(opt: &mut OptionsPile<SortOptions>, aliases: &[&str], help: impl ToOptionsHelp) {
        opt.add_sub(|p| &mut (p.0).0, sorts::REGISTRY.single_options(aliases, help));
        opt.add_sub(|p| &mut (p.0).0, sorts::REGISTRY.multiple_options(aliases));
    }

    pub fn help_options<X: 'static>() -> OptionsPile<X> {
        return sorts::REGISTRY.help_options("sort");
    }

    pub fn new_options(aliases: &[&str], help: impl ToOptionsHelp) -> OptionsPile<SortOptions> {
        let mut opt = OptionsPile::new();
        Self::options(&mut opt, aliases, help);
        return opt;
    }

    pub fn push(&mut self, s: BoxedSort) {
        (self.0).0.push(s);
    }
}

pub struct GenericSortBucket<T> {
    ts: HashMap<usize, T>,
    i: usize,
    bucket: Box<dyn SortBucket>,
}

impl<T> GenericSortBucket<T> {
    pub fn add(&mut self, r: Record, t: T) {
        let i = self.i;
        self.i += 1;

        self.ts.insert(i, t);
        self.bucket.add(r, i);
    }

    fn removed(&mut self, e: Option<(Record, usize)>) -> Option<(Record, T)> {
        return match e {
            Some((r, i)) => Some((r, self.ts.remove(&i).unwrap())),
            None => None,
        };
    }

    pub fn remove_last(&mut self) -> Option<(Record, T)> {
        let e = self.bucket.remove_last();
        return self.removed(e);
    }

    pub fn remove_first(&mut self) -> Option<(Record, T)> {
        let e = self.bucket.remove_first();
        return self.removed(e);
    }

    pub fn size(&self) -> usize {
        return self.ts.len();
    }
}

impl SortOptionsValidated {
    pub fn new_bucket<T>(&self) -> GenericSortBucket<T> {
        let f: Rc<dyn Fn() -> Box<dyn SortBucket>> = Rc::new(VecDequeSortBucket::new);

        let f = self.0.iter().rev().fold(f, |f, sort| {
            let sort = sort.clone();
            return Rc::new(move || sort.new_bucket(f.clone()));
        });

        return GenericSortBucket {
            ts: HashMap::new(),
            i: 0,
            bucket: f(),
        };
    }
}
