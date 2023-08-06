#![allow(unused_imports)]
use nalgebra as na;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;

use super::general_graph::GeneralUndiGraph;

pub struct NormalUndiGraph {
    pub name: String,
    pub n: usize,
    pub m: usize,
    pub adjs: Vec<Vec<usize>>,
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
    pub fn diag_adj(&self) -> (na::DVector<f64>, na::DMatrix<f64>) {
        let diag_vec: na::DVector<f64> =
            na::DVector::from_iterator(self.n, self.adjs.iter().map(|adj| adj.len() as f64));
        let mut adj_mat: na::DMatrix<f64> = na::DMatrix::zeros(self.n, self.n);
        for (u, adj) in self.adjs.iter().enumerate() {
            let i = adj.partition_point(|v| v <= &u);
            for &v in adj[i..].iter() {
                adj_mat[(u, v)] += 1f64;
                adj_mat[(v, u)] += 1f64;
            }
        }
        return (diag_vec, adj_mat);
    }
    pub fn from_general(g: &GeneralUndiGraph) -> Self {
        let n = g.num_nodes();
        if n == 0 {
            return Self {
                name: "EmptyGraph".to_string(),
                n: 0,
                m: 0,
                adjs: Vec::new(),
            };
        }
        let mut degs = vec![0usize; n];
        let mut o2n: HashMap<usize, usize> = HashMap::new();
        let renumber = g.nodes.iter().max().unwrap() + 1 != n;
        if renumber {
            for &(u, v) in &g.edges {
                let tot = o2n.len();
                let &mut new_u = o2n.entry(u).or_insert(tot);
                let tot = o2n.len();
                let &mut new_v = o2n.entry(v).or_insert(tot);
                degs[new_u] += 1;
                degs[new_v] += 1;
            }
        } else {
            for &(u, v) in &g.edges {
                degs[u] += 1;
                degs[v] += 1;
            }
        }
        let mut adjs: Vec<Vec<usize>> = Vec::with_capacity(n);
        for u in 0..n {
            adjs.push(Vec::with_capacity(degs[u]));
        }
        if renumber {
            for &(u, v) in &g.edges {
                let (new_u, new_v) = (o2n[&u], o2n[&v]);
                adjs[new_u].push(new_v);
                adjs[new_v].push(new_u);
            }
        } else {
            for &(u, v) in &g.edges {
                adjs[u].push(v);
                adjs[v].push(u);
            }
        }
        for u in 0..n {
            adjs[u].sort_unstable();
        }
        Self {
            name: g.name.clone(),
            n,
            m: g.num_edges(),
            adjs,
        }
    }
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

    #[test]
    fn test_konect_euro() {
        use super::super::general_graph::GeneralUndiGraph;
        let mut wf = File::create("test_konect_euro_normal.txt").unwrap();
        let g = GeneralUndiGraph::from_konect("euro", "subelj_euroroad")
            .unwrap()
            .lcc();
        let g = NormalUndiGraph::from_general(&g);
        write!(wf, "{}", g).unwrap();
    }

    #[test]
    fn test_diag_adj() {
        use super::super::general_graph::GeneralUndiGraph;
        let path = "test_diag_adj_input.txt";
        let mut wf = File::create(path).unwrap();
        write!(wf, "0 1\n0 2\n0 3\n1 3\n").unwrap();
        let rf = File::open(path).unwrap();
        let g = GeneralUndiGraph::from_file("test_diag_adj", rf);
        std::fs::remove_file(path).unwrap();
        let g = NormalUndiGraph::from_general(&g);
        let (diag, adj) = g.diag_adj();
        let lap = na::DMatrix::from_diagonal(&diag) - adj;
        let mut wf = File::create("test_diag_adj_output.txt").unwrap();
        writeln!(wf, "lap:\n{}", lap).unwrap();
    }
}
