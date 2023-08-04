#![allow(unused_imports)]
use std::fmt;
use std::io::Write;

pub struct NormalUndiGraph {
    name: String,
    n: usize,
    m: usize,
    adjs: Vec<Vec<usize>>,
}

impl fmt::Display for NormalUndiGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "# NormalUndiGraph: {}\n# Nodes: {} Edges: {}\n",
            self.name, self.n, self.m
        )?;
        for (u, adj) in self.adjs.iter().enumerate() {
            let i = adj.partition_point(|v| v <= &u);
            for v in adj[i..].iter() {
                writeln!(f, "{}\t{}", u, v)?;
            }
        }
        Ok(())
    }
}

impl NormalUndiGraph {
    pub fn from_apollo(g: usize) -> Self {
        let mut triangles: Vec<(usize, usize, usize)> =
            vec![(0, 1, 2), (0, 1, 3), (0, 2, 3), (1, 2, 3)];
        let mut active_triangles = triangles.clone();
        let mut adjs: Vec<Vec<usize>> =
            vec![vec![1, 2, 3], vec![0, 2, 3], vec![0, 1, 3], vec![0, 1, 2]];
        let mut n: usize = 4;
        let mut m: usize = 6;
        for _ in 0..g {
            let mut new_triangles = Vec::with_capacity(3 * active_triangles.len());
            for &(x, y, z) in &active_triangles {
                new_triangles.append(&mut vec![(x, y, n), (x, z, n), (y, z, n)]);
                adjs[x].push(n);
                adjs[y].push(n);
                adjs[z].push(n);
                adjs.push(vec![x, y, z]);
                n += 1;
            }
            m += new_triangles.len();
            triangles.append(&mut new_triangles.clone());
            active_triangles = new_triangles;
        }
        Self {
            name: format!("Apollo_{}", g),
            n,
            m,
            adjs,
        }
    }
    pub fn from_koch(g: usize) -> Self {
        let mut triangles: Vec<(usize, usize, usize)> = vec![(0, 1, 2)];
        let mut n: usize = 3;
        for _ in 0..g {
            let mut new_triangles = Vec::with_capacity(3 * triangles.len());
            for &(x, y, z) in &triangles {
                new_triangles.push((x, n, n + 1));
                new_triangles.push((y, n + 2, n + 3));
                new_triangles.push((z, n + 4, n + 5));
                n += 6;
            }
            triangles.append(&mut new_triangles);
        }
        let mut adjs: Vec<Vec<usize>> = Vec::new();
        adjs.resize_with(n, Default::default);
        for &(x, y, z) in &triangles {
            adjs[x].append(&mut vec![y, z]);
            adjs[y].append(&mut vec![x, z]);
            adjs[z].append(&mut vec![x, y]);
        }
        Self {
            name: format!("Koch_{}", g),
            n,
            m: 3 * triangles.len(),
            adjs,
        }
    }
    fn _from_pseudo_ext(m: usize, g: usize, name: String) -> Self {
        let mut edges: Vec<(usize, usize)> = vec![(0, 1), (0, 2), (1, 2)];
        let mut n: usize = 3;
        for _ in 0..g {
            let mut new_edges = Vec::with_capacity(2 * m * edges.len());
            for &(u, v) in &edges {
                for _ in 0..m {
                    new_edges.push((u, n));
                    new_edges.push((v, n));
                    n += 1;
                }
            }
            edges.append(&mut new_edges);
        }
        let mut adjs: Vec<Vec<usize>> = Vec::new();
        adjs.resize_with(n, Default::default);
        for &(u, v) in &edges {
            adjs[u].push(v);
            adjs[v].push(u);
        }
        Self {
            name,
            n,
            m: edges.len(),
            adjs,
        }
    }
    pub fn from_pseudo_ext(m: usize, g: usize) -> Self {
        Self::_from_pseudo_ext(m, g, format!("PseudoExt_{}_{}", m, g))
    }
    pub fn from_pseudofractal(g: usize) -> Self {
        Self::_from_pseudo_ext(1, g, format!("Pseudofractal_{}", g))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn test_pseudo_ext() {
        let mut wf = File::create("pseudoext_2_4.txt").unwrap();
        let g = NormalUndiGraph::from_pseudo_ext(2, 4);
        write!(wf, "{}", g).unwrap();
    }

    #[test]
    fn test_koch() {
        let mut wf = File::create("koch_4.txt").unwrap();
        let g = NormalUndiGraph::from_koch(4);
        write!(wf, "{}", g).unwrap();
    }

    #[test]
    fn test_apollo() {
        let mut wf = File::create("apollo_4.txt").unwrap();
        let g = NormalUndiGraph::from_apollo(4);
        write!(wf, "{}", g).unwrap();
    }
}
