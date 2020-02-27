use record::Record;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::rc::Rc;

pub enum SortBucketSide {
    Front(),
    Back(),
}

impl SortBucketSide {
    fn next<T, I: DoubleEndedIterator<Item = T>>(&self, i: &mut I) -> Option<T> {
        return match self {
            SortBucketSide::Front() => i.next(),
            SortBucketSide::Back() => i.next_back(),
        };
    }
}

pub trait SortBucket {
    fn add(&mut self, r: Record, i: usize);
    fn remove_from(&mut self, side: SortBucketSide) -> Option<(Record, usize)>;
    fn is_empty(&self) -> bool;

    fn remove_last(&mut self) -> Option<(Record, usize)> {
        return self.remove_from(SortBucketSide::Back());
    }

    fn remove_first(&mut self) -> Option<(Record, usize)> {
        return self.remove_from(SortBucketSide::Front());
    }
}

pub struct KeySortBucket<T: Clone + Ord, F: Fn(Record, usize) -> T> {
    f: F,
    next: Rc<dyn Fn() -> Box<dyn SortBucket>>,
    map: BTreeMap<T, Box<dyn SortBucket>>,
}

impl<T: Clone + Ord, F: Fn(Record, usize) -> T> SortBucket for KeySortBucket<T, F> {
    fn add(&mut self, r: Record, i: usize) {
        let t = (self.f)(r.clone(), i);
        let next = &self.next;
        self.map.entry(t).or_insert_with(|| next()).add(r, i);
    }

    fn remove_from(&mut self, side: SortBucketSide) -> Option<(Record, usize)> {
        let t = match side.next(&mut self.map.keys()) {
            Some(t) => t.clone(),
            None => return None,
        };

        let mut next = self.map.remove(&t).unwrap();
        assert!(!next.is_empty());

        let ret = next.remove_from(side);
        assert!(ret.is_some());

        if !next.is_empty() {
            self.map.insert(t, next);
        }

        return ret;
    }

    fn is_empty(&self) -> bool {
        return self.map.is_empty();
    }
}

impl<T: Clone + Ord + 'static, F: Fn(Record, usize) -> T + 'static> KeySortBucket<T, F> {
    pub fn new(f: F, next: Rc<dyn Fn() -> Box<dyn SortBucket>>) -> Box<dyn SortBucket> {
        return Box::new(KeySortBucket {
            f: f,
            next: next,
            map: BTreeMap::new(),
        });
    }
}

#[derive(Default)]
pub struct VecDequeSortBucket(VecDeque<(Record, usize)>);

impl SortBucket for VecDequeSortBucket {
    fn add(&mut self, r: Record, i: usize) {
        self.0.push_back((r, i));
    }

    fn remove_from(&mut self, side: SortBucketSide) -> Option<(Record, usize)> {
        return match side {
            SortBucketSide::Front() => self.0.pop_front(),
            SortBucketSide::Back() => self.0.pop_back(),
        };
    }

    fn is_empty(&self) -> bool {
        return self.0.is_empty();
    }
}

impl VecDequeSortBucket {
    pub fn new() -> Box<dyn SortBucket> {
        return Box::new(VecDequeSortBucket::default());
    }
}
