#![allow(dead_code)]
use std::collections::HashMap;

struct DSUEntry<T>
where
    T: std::cmp::Eq + std::hash::Hash + Clone,
{
    val: T,
    parent: usize,
}

pub struct DSU<T>
where
    T: std::cmp::Eq + std::hash::Hash + Clone,
{
    entries: Vec<DSUEntry<T>>,
    indices: HashMap<T, usize>,
}
impl<T> DSU<T>
where
    T: std::cmp::Eq + std::hash::Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            indices: HashMap::new(),
        }
    }
    pub fn add_unchecked(&mut self, val: T) -> usize {
        let i = self.entries.len();
        self.entries.push(DSUEntry {
            val: val.clone(),
            parent: i,
        });
        self.indices.insert(val, i);
        i
    }
    pub fn add(&mut self, val: T) -> bool {
        if self.indices.contains_key(&val) {
            false
        } else {
            self.add_unchecked(val);
            true
        }
    }
    fn index(&mut self, val: T) -> usize {
        match self.indices.get(&val) {
            Some(i) => i.clone(),
            None => self.add_unchecked(val),
        }
    }
    fn find_by_index(&mut self, x: usize) -> usize {
        let px = self.entries[x].parent;
        if px == x {
            px
        } else {
            let px = self.find_by_index(px);
            self.entries[x].parent = px;
            px
        }
    }
    pub fn find_unchecked(&mut self, val: T) -> usize {
        let x = self.indices[&val];
        self.find_by_index(x)
    }
    pub fn find(&mut self, val: T) -> usize {
        let x = self.index(val);
        self.find_by_index(x)
    }
    pub fn union_unchecked(&mut self, val_x: T, val_y: T) -> bool {
        let (px, py) = (self.find_unchecked(val_x), self.find_unchecked(val_y));
        self.entries[px].parent = py;
        px != py
    }
    pub fn union(&mut self, val_x: T, val_y: T) -> bool {
        let (px, py) = (self.find(val_x), self.find(val_y));
        self.entries[px].parent = py;
        px != py
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut dsu: DSU<&str> = DSU::new();
        assert_eq!(dsu.add("1"), true);
        assert_eq!(dsu.add("2"), true);
        assert_eq!(dsu.add("1"), false);
        assert_eq!(dsu.add("2"), false);
    }

    #[test]
    fn test_union() {
        let mut dsu: DSU<&str> = DSU::new();
        assert_ne!(dsu.find("1"), dsu.find("2"));
        assert_ne!(dsu.find("1"), dsu.find("3"));
        assert_ne!(dsu.find("2"), dsu.find("3"));
        dsu.union("1", "2");
        assert_eq!(dsu.find("1"), dsu.find("2"));
        assert_ne!(dsu.find("1"), dsu.find("3"));
        assert_ne!(dsu.find("2"), dsu.find("3"));
    }

    #[test]
    fn test_union_unchecked() {
        let mut dsu: DSU<&str> = DSU::new();
        dsu.add_unchecked("1");
        dsu.add_unchecked("2");
        dsu.add_unchecked("3");
        assert_ne!(dsu.find_unchecked("1"), dsu.find_unchecked("2"));
        assert_ne!(dsu.find_unchecked("1"), dsu.find_unchecked("3"));
        assert_ne!(dsu.find_unchecked("2"), dsu.find_unchecked("3"));
        dsu.union_unchecked("1", "2");
        assert_eq!(dsu.find_unchecked("1"), dsu.find_unchecked("2"));
        assert_ne!(dsu.find_unchecked("1"), dsu.find_unchecked("3"));
        assert_ne!(dsu.find_unchecked("2"), dsu.find_unchecked("3"));
    }
}
