use bytes::BufMut;
use bzip2::read::BzDecoder;
use futures::StreamExt;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{read_dir, remove_dir, File};
use std::io::{BufRead, BufReader, Write};
use std::time::Instant;
use tar::Archive;
use tempfile::Builder;

enum FastDSUEntry {
    Id(usize),
    Num(i64),
}

struct FastDSU {
    parent: HashMap<usize, FastDSUEntry>,
}

use FastDSUEntry::*;

impl FastDSU {
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
        }
    }
    fn add(&mut self, x: usize) {
        self.parent.insert(x, Num(1));
    }
    fn find(&self, x: &usize) -> usize {
        match self.parent[x] {
            Num(_) => *x,
            Id(px) => self.find(&px),
        }
    }
    fn find_all(&mut self, x: usize) -> (usize, i64) {
        match self.parent[&x] {
            Num(num) => (x, num),
            Id(px) => {
                let (px, num) = self.find_all(px);
                self.parent.insert(x, Id(px));
                (px, num)
            }
        }
    }
    fn union(&mut self, x: usize, y: usize) -> bool {
        // let (px, py) = (self.find(x), self.find(y));
        let (x, x_num) = self.find_all(x);
        let (y, y_num) = self.find_all(y);
        if x != y {
            if x_num < y_num {
                self.parent.insert(x, Id(y));
                self.parent.insert(y, Num(x_num + y_num));
            } else {
                self.parent.insert(y, Id(x));
                self.parent.insert(x, Num(x_num + y_num));
            }
            true
        } else {
            false
        }
    }
    fn retain_map(&self) -> HashMap<usize, bool> {
        let (root, _) = self
            .parent
            .iter()
            .max_by_key(|(_, y)| match y {
                Id(_) => &-1,
                Num(num) => num,
            })
            .unwrap();
        let root = *root;
        let retain_iter = self.parent.keys().map(|x| (*x, self.find(x) == root));
        let retain_vec: Vec<(usize, bool)> = retain_iter.clone().collect();
        let mut wf = File::create("retain_iter.txt").unwrap();
        write!(wf, "{:?}", retain_vec).unwrap();
        HashMap::from_iter(retain_iter)
    }
}

async fn fetch_raw_bytes(url: &str) -> Result<Vec<u8>, String> {
    let start = Instant::now();
    let resp = reqwest::get(url)
        .await
        .or(Err(format!("Failed to GET from '{}'", url)))?;
    let total_size = resp
        .content_length()
        .ok_or(format!("Failed to fetch content length from '{}'", url))?;
    let sty=ProgressStyle::with_template(
        "{msg} {wide_bar:.cyan/blue} {bytes}/{total_bytes} {bytes_per_sec} [{elapsed_precise}/{eta_precise}]"
    ).or(Err("Failed to generate progess style template"))?.progress_chars("##=");
    let pb = ProgressBar::new(total_size)
        .with_style(sty)
        .with_message(format!("Fetching {}", url));

    let mut payload = Vec::with_capacity(total_size as usize);
    let mut fetched_size: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.or(Err(format!("Error while fetching payload of '{}'", url)))?;
        payload.put(&chunk[..]);
        fetched_size = total_size.min(fetched_size + chunk.len() as u64);
        pb.set_position(fetched_size);
    }

    pb.println(format!(
        "Fetched {} in {}",
        url,
        HumanDuration(start.elapsed())
    ));
    pb.finish_and_clear();
    Ok(payload)
}

pub struct GeneralUndiGraph {
    pub name: String,
    pub nodes: HashSet<usize>,
    pub edges: HashSet<(usize, usize)>,
}

impl fmt::Display for GeneralUndiGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "# GeneralUndiGraph: {}\n# Nodes: {} Edges: {}\n",
            self.name,
            self.num_nodes(),
            self.num_edges()
        )?;
        let mut edges: Vec<&(usize, usize)> = self.edges.iter().collect();
        edges.sort_unstable();
        for (u, v) in edges {
            writeln!(f, "{}\t{}", u, v)?;
        }
        Ok(())
    }
}

impl GeneralUndiGraph {
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }
    pub fn add_edge(&mut self, u: usize, v: usize) {
        if u == v {
            return;
        }
        self.nodes.insert(u);
        self.nodes.insert(v);
        if u < v {
            self.edges.insert((u, v));
        } else {
            self.edges.insert((v, u));
        }
    }
    pub fn new(name: String) -> Self {
        Self {
            name,
            nodes: HashSet::new(),
            edges: HashSet::new(),
        }
    }
    #[tokio::main]
    pub async fn from_konect(name: &str, internal_name: &str) -> Result<Self, String> {
        let url = format!(
            "http://konect.cc/files/download.tsv.{}.tar.bz2",
            internal_name
        );
        let tarbz2_bytes = fetch_raw_bytes(&url).await?;
        let bzdecoder = BzDecoder::new(tarbz2_bytes.as_slice());
        let mut archive = Archive::new(bzdecoder);
        let tmp_dir = Builder::new()
            .tempdir()
            .or(Err("Failed to return a temp dir"))?;
        let tmp_dir = tmp_dir.path();
        let dir_path = tmp_dir.join(internal_name);
        // println!("Decompressed file will be located under {:?}", &dir_path);
        if dir_path.try_exists().unwrap_or(false) {
            remove_dir(&dir_path).or(Err("Failed to remove temp dir"))?;
        }
        archive.unpack(tmp_dir).or(Err(format!(
            "Failed to unpack tarball of '{}'",
            internal_name
        )))?;
        for entry in read_dir(&dir_path).or(Err("Failed to traverse content of temp dir"))? {
            let file_path = entry
                .or(Err("Failed to traverse entry of temp dir"))?
                .path();
            if let Some(file_name) = file_path.file_name().and_then(|name| name.to_str()) {
                if file_name.starts_with("out.") {
                    let f = File::open(file_path).or(Err("Failed to open konect file"))?;
                    return Ok(Self::from_file(name, f));
                }
            }
        }
        Err("Failed to find valid konect file in extracted dir".to_string())
    }
    pub fn from_file(name: &str, f: File) -> Self {
        let mut g = Self::new(name.to_string());
        let reader = BufReader::new(f);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.starts_with("#") || line.starts_with("%") {
                continue;
            }
            let mut split = line.split(&[' ', '\t']);
            let u: usize = split.next().unwrap().parse().unwrap();
            let v: usize = split.next().unwrap().parse().unwrap();
            g.add_edge(u, v);
        }
        g
    }
    pub fn lcc(mut self) -> Self {
        let mut dsu = FastDSU::new();
        for u in &self.nodes {
            dsu.add(*u);
        }
        for (u, v) in &self.edges {
            dsu.union(*u, *v);
        }
        let rmap = dsu.retain_map();
        self.nodes.retain(|u| rmap[u]);
        self.edges.retain(|(u, v)| rmap[u] && rmap[v]);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disjoint_set() {
        let rf = File::open("subelj_euroroad.txt").unwrap();
        let mut wf = File::create("euro.txt").unwrap();
        let g = GeneralUndiGraph::from_file("euro", rf).lcc();
        write!(wf, "{}", g).unwrap();
    }

    #[test]
    fn test_konect_euro() {
        let g = GeneralUndiGraph::from_konect("euro", "subelj_euroroad").unwrap();
        let mut wf = File::create("test_konect_euro.txt").unwrap();
        write!(wf, "{}", g).unwrap();
    }
}
