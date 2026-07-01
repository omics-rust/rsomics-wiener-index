//! Criterion bench: compute Wiener index on a 600-node, 3000-edge connected graph.
//!
//! The fixture lives at /Volumes/KIOXIA/tmp/bench_600.txt (generated once).
//! On machines without the fixture, the bench is skipped.

use criterion::{Criterion, criterion_group, criterion_main};
use std::collections::{HashMap, VecDeque};
use std::hint::black_box;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn parse(path: &Path) -> (Vec<Vec<usize>>, usize) {
    let f = std::fs::File::open(path).expect("open fixture");
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut adj: Vec<Vec<usize>> = Vec::new();

    for line in BufReader::new(f).lines() {
        let line = line.unwrap();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_ascii_whitespace();
        let u_s = parts.next().unwrap();
        let v_s = parts.next().unwrap();
        let u = intern(u_s, &mut index, &mut adj);
        let v = intern(v_s, &mut index, &mut adj);
        if u != v && !adj[u].contains(&v) {
            adj[u].push(v);
            adj[v].push(u);
        }
    }
    let n = adj.len();
    (adj, n)
}

fn intern(name: &str, index: &mut HashMap<String, usize>, adj: &mut Vec<Vec<usize>>) -> usize {
    if let Some(&id) = index.get(name) {
        return id;
    }
    let id = adj.len();
    index.insert(name.to_owned(), id);
    adj.push(Vec::new());
    id
}

fn wiener_compute(adj: &[Vec<usize>], n: usize) -> f64 {
    let mut total: u64 = 0;
    let mut dist = vec![u64::MAX; n];
    let mut queue = VecDeque::new();

    for src in 0..n {
        dist.fill(u64::MAX);
        dist[src] = 0;
        queue.clear();
        queue.push_back(src);
        let mut reached = 1usize;

        while let Some(u) = queue.pop_front() {
            for &v in &adj[u] {
                if dist[v] == u64::MAX {
                    dist[v] = dist[u] + 1;
                    reached += 1;
                    queue.push_back(v);
                }
            }
        }

        if reached < n {
            return f64::INFINITY;
        }

        total += dist[src + 1..].iter().sum::<u64>();
    }

    total as f64
}

fn bench_wiener(c: &mut Criterion) {
    let fixture = Path::new("/Volumes/KIOXIA/tmp/bench_600.txt");
    if !fixture.exists() {
        eprintln!("bench fixture not found, skipping");
        return;
    }

    let (adj, n) = parse(fixture);

    c.bench_function("wiener_600node", |b| {
        b.iter(|| wiener_compute(black_box(&adj), black_box(n)));
    });
}

criterion_group!(benches, bench_wiener);
criterion_main!(benches);
