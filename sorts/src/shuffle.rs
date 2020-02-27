use rand::Rng;
use registry::args::ZeroRegistryArgs;
use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Mutex;
use super::SortBe;
use super::SortRegistrant;
use super::bucket::KeySortBucket;
use super::bucket::SortBucket;

#[derive(Clone)]
#[derive(Default)]
struct RandomSortKey(usize, Rc<Mutex<Vec<u8>>>);

impl PartialEq for RandomSortKey {
    fn eq(&self, other: &Self) -> bool {
        return self.0 == other.0;
    }
}

impl Eq for RandomSortKey {
}

impl PartialOrd for RandomSortKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

impl Ord for RandomSortKey {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 == other.0 {
            return Ordering::Equal;
        }

        let mut i = 0;
        loop {
            let r = self.at(i).cmp(&other.at(i));
            if let Ordering::Equal = r {
                i += 1;
                continue;
            }
            return r;
        }
    }
}

impl RandomSortKey {
    fn new(i: usize) -> Self {
        return RandomSortKey(i, Rc::new(Mutex::new(Vec::new())));
    }

    fn at(&self, i: usize) -> u8 {
        let mut mg = self.1.lock().unwrap();
        while i >= mg.len() {
            mg.push(rand::thread_rng().gen());
        }
        return mg[i];
    }
}

pub type Impl = SortRegistrant<ImplBe>;

pub struct ImplBe;

impl SortBe for ImplBe {
    type Args = ZeroRegistryArgs;

    fn names() -> Vec<&'static str> {
        return vec!["shuffle"];
    }

    fn help_msg() -> &'static str {
        return "'sort' randomly";
    }

    fn new_bucket(_a: &ZeroRegistryArgs, next: Rc<dyn Fn() -> Box<dyn SortBucket>>) -> Box<dyn SortBucket> {
        return KeySortBucket::new(|_r, i| RandomSortKey::new(i), next);
    }
}
