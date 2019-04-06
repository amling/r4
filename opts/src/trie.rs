use std::collections::HashMap;

pub struct NameTrie<T: Eq> {
    own: Option<(String, T)>,
    children: HashMap<char, NameTrie<T>>,
}

impl<T: Eq> Default for NameTrie<T> {
    fn default() -> Self {
        return NameTrie {
            own: None,
            children: HashMap::new(),
        };
    }
}

pub enum NameTrieResult<'a, T: Eq> {
    None(),
    Unique(&'a str, &'a T),
    Collision(&'a str, &'a str),
}

impl<'a, T: Eq> NameTrieResult<'a, T> {
    fn add(&mut self, name: &'a str, t: &'a T) -> bool {
        *self = match *self {
            NameTrieResult::None() => NameTrieResult::Unique(name, t),
            NameTrieResult::Unique(name1, t1) => if t == t1 { NameTrieResult::Unique(name1, t1) } else { NameTrieResult::Collision(name1, name) },
            NameTrieResult::Collision(name1, name2) => NameTrieResult::Collision(name1, name2),
        };
        if let NameTrieResult::Collision(_, _) = self {
            return true;
        }
        return false;
    }
}

impl<T: Eq> NameTrie<T> {
    pub fn get<'a>(&'a self, name: &str) -> NameTrieResult<'a, T> {
        let mut n = self;
        for c in name.chars() {
            match n.children.get(&c) {
                None => {
                    return NameTrieResult::None();
                }
                Some(ref n2) => {
                    n = n2;
                }
            }
        }
        if let Some((ref name, ref t)) = n.own {
            // Exact, honor even if there are longer matches.
            return NameTrieResult::Unique(name, t);
        }
        let mut acc = NameTrieResult::None();
        n.collect(&mut acc);
        return acc;
    }

    fn collect<'a>(&'a self, acc: &mut NameTrieResult<'a, T>) -> bool {
        if let Some((ref name, ref t)) = self.own {
            if acc.add(name, t) {
                return true;
            }
        }
        for n in self.children.values() {
            if n.collect(acc) {
                return true;
            }
        }
        return false;
    }

    pub fn insert(&mut self, name: &str, t: T) {
        let n = name.chars().fold(self, |n, c| n.children.entry(c).or_insert_with(NameTrie::default));
        if n.own.is_some() {
            panic!("Collision in options at {}", name);
        }
        n.own = Some((name.to_string(), t));
    }
}
