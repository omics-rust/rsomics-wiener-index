use clap::Parser;
use std::collections::{HashMap, VecDeque};
use std::io::{self, BufRead};

#[derive(Parser)]
#[command(
    name = "rsomics-wiener-index",
    about = "Wiener index: sum of shortest-path distances over all unordered node pairs"
)]
struct Cli {
    /// Output as JSON object with key "wiener_index"
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    let result = run();
    if cli.json {
        match result {
            Some(w) if w.is_infinite() => {
                println!("{{\"wiener_index\":null,\"disconnected\":true}}")
            }
            Some(w) => println!("{{\"wiener_index\":{w}}}"),
            None => println!("{{\"wiener_index\":0}}"),
        }
    } else {
        match result {
            Some(w) if w.is_infinite() => println!("inf"),
            Some(w) => println!("{w}"),
            None => println!("0"),
        }
    }
}

fn run() -> Option<f64> {
    let stdin = io::stdin();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut adj: Vec<Vec<usize>> = Vec::new();

    for line in stdin.lock().lines() {
        let line = line.expect("stdin read error");
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // nx.parse_edgelist skips lines with fewer than 2 whitespace tokens.
        let mut parts = trimmed.split_ascii_whitespace();
        let (Some(u_str), Some(v_str)) = (parts.next(), parts.next()) else {
            continue;
        };

        let u = intern(u_str, &mut index, &mut adj);
        let v = intern(v_str, &mut index, &mut adj);

        // nx.Graph silently deduplicates parallel edges; mirror that.
        if u != v && !adj[u].contains(&v) {
            adj[u].push(v);
            adj[v].push(u);
        }
    }

    let n = adj.len();
    if n == 0 {
        return Some(0.0);
    }

    // W = Σ_{unordered pairs} d(u,v)  [= (1/2) Σ_{ordered pairs}, matching nx for undirected]
    // Any pair unreachable → W = ∞.
    let mut total: u64 = 0;
    let mut dist = vec![u64::MAX; n];

    for src in 0..n {
        dist.fill(u64::MAX);
        dist[src] = 0;
        let mut queue = VecDeque::new();
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
            return Some(f64::INFINITY);
        }

        // Only sum pairs (src, v) with v > src to count each unordered pair once.
        total += dist[src + 1..].iter().sum::<u64>();
    }

    Some(total as f64)
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

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
