// src/audit/union_find.rs
//! Union-Find data structure for clustering.

pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

// Indexing is safe here: all indices passed to find/union must be < n (from new()).
// This is a classic data structure where indices are guaranteed valid by construction.
#[allow(clippy::indexing_slicing)]
impl UnionFind {
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    pub fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);

        if rx == ry {
            return;
        }

        match self.rank[rx].cmp(&self.rank[ry]) {
            std::cmp::Ordering::Less => self.parent[rx] = ry,
            std::cmp::Ordering::Greater => self.parent[ry] = rx,
            std::cmp::Ordering::Equal => {
                self.parent[ry] = rx;
                self.rank[rx] += 1;
            }
        }
    }
}
