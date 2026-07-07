//! Compatibility tests against networkx.wiener_index (BSD-3-Clause).
//!
//! Oracle values were produced with networkx 3.6.1:
//!   python3 -c "import networkx as nx; ..."
//! All expected values are hardcoded — never derived from our own binary.

use std::io::Write;
use std::process::{Command, Stdio};

fn binary() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop(); // deps/
    p.pop(); // debug/ or release/
    p.push("rsomics-wiener-index");
    p
}

fn run(input: &str) -> String {
    let mut child = Command::new(binary())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn binary");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "binary exited non-zero");
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

fn run_json(input: &str) -> serde_json::Value {
    let mut child = Command::new(binary())
        .arg("--json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn binary");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    serde_json::from_slice(&out.stdout).unwrap()
}

// Edge list helpers
fn path_edges(n: usize) -> String {
    (0..n - 1).map(|i| format!("{} {}\n", i, i + 1)).collect()
}

fn cycle_edges(n: usize) -> String {
    let mut s: String = (0..n - 1).map(|i| format!("{} {}\n", i, i + 1)).collect();
    s.push_str(&format!("{} 0\n", n - 1));
    s
}

fn complete_edges(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n {
        for j in i + 1..n {
            s.push_str(&format!("{i} {j}\n"));
        }
    }
    s
}

// --- Hand graphs (formula-verified against oracle) ---

/// path_graph(4): oracle = 10.0
#[test]
fn path4() {
    assert_eq!(run(&path_edges(4)), "10");
}

/// path_graph(6): oracle = 35.0
#[test]
fn path6() {
    assert_eq!(run(&path_edges(6)), "35");
}

/// cycle_graph(4): oracle = 8.0
#[test]
fn cycle4() {
    assert_eq!(run(&cycle_edges(4)), "8");
}

/// complete_graph(4): oracle = 6.0
#[test]
fn complete4() {
    assert_eq!(run(&complete_edges(4)), "6");
}

/// star_graph(3): oracle = 9.0
/// Edges: 0-1, 0-2, 0-3 (node 0 is center)
#[test]
fn star3() {
    let input = "0 1\n0 2\n0 3\n";
    assert_eq!(run(input), "9");
}

/// K2: oracle = 1.0
#[test]
fn k2() {
    assert_eq!(run("0 1\n"), "1");
}

/// Single node (no edges): oracle = 0.0
#[test]
fn single_node() {
    // Feed a comment line so we get a 1-node graph — but actually a single
    // node with no edges produces 0 pairs → W=0. We test by having no edges.
    // An empty stdin = 0 nodes = W=0 as well. Let's just confirm.
    assert_eq!(run(""), "0");
}

// --- Disconnected → inf ---

/// Two disconnected edges: oracle = inf
#[test]
fn disconnected_two_components() {
    let input = "0 1\n2 3\n";
    assert_eq!(run(input), "inf");
}

/// Disconnected with string nodes: oracle = inf
#[test]
fn disconnected_strings() {
    let input = "a b\nc d\n";
    assert_eq!(run(input), "inf");
}

// --- Input format: comments and blanks are skipped ---

#[test]
fn skip_comments_and_blanks() {
    // 3 edges → path_graph(4) (nodes 0,1,2,3): oracle = 10.0
    let input = "# this is a comment\n\n0 1\n\n# another comment\n1 2\n2 3\n";
    assert_eq!(run(input), "10");
}

// --- Stray single-token lines are skipped (nx.parse_edgelist behaviour) ---

/// A line with fewer than 2 whitespace tokens (a bare 'X') is skipped, exactly
/// like nx.parse_edgelist. The remaining 3 edges form path_graph(4): oracle = 10.0.
/// Verified: nx.parse_edgelist(['0 1','1 2','2 3','X']) → wiener_index = 10.0.
#[test]
fn skip_stray_single_token_line() {
    assert_eq!(run("0 1\n1 2\n2 3\nX\n"), "10");
}

/// Stray token line in the middle is also skipped: oracle = 10.0.
#[test]
fn skip_stray_single_token_line_mid() {
    assert_eq!(run("0 1\nX\n1 2\n2 3\n"), "10");
}

// --- Parallel edge dedup (nx.Graph silently ignores) ---

#[test]
fn parallel_edge_dedup() {
    // Duplicate edge 0-1; result should be same as path P3: oracle = 4.0
    let input = "0 1\n0 1\n1 2\n";
    assert_eq!(run(input), "4");
}

// --- Random graph: gnm(10,20,seed=42) oracle = 74.0 ---
// Verified with:
//   nx.generators.random_graphs.gnm_random_graph(10, 20, seed=42) → connected → wiener=74.0

#[test]
fn gnm_10_20_seed42() {
    let input = "\
0 1
0 4
0 6
0 8
1 3
1 5
1 6
1 8
1 9
2 3
2 6
3 4
3 6
3 8
4 5
4 9
5 9
7 8
7 9
8 9
";
    assert_eq!(run(input), "74");
}

// --- Random graph: largest_component(20,40,seed=123) oracle = 433.0 ---

#[test]
fn gnm_20_40_seed123_largest_component() {
    let input = "\
0 9
0 10
0 11
0 13
1 2
1 5
1 8
1 9
1 12
1 15
2 11
2 12
2 13
2 14
2 15
2 19
3 8
3 14
4 10
5 7
5 8
6 18
6 19
7 12
7 15
8 15
9 10
10 17
11 16
12 16
12 17
12 18
13 16
13 17
13 18
15 16
15 17
15 19
16 17
16 18
";
    assert_eq!(run(input), "433");
}

// --- connected_caveman_graph(3,4) oracle = 153.0 ---
// Edges: sorted(nx.connected_caveman_graph(3,4).edges())

#[test]
fn caveman_3_4() {
    let input = "\
0 2
0 3
0 11
1 2
1 3
2 3
3 4
4 6
4 7
5 6
5 7
6 7
7 8
8 10
8 11
9 10
9 11
10 11
";
    assert_eq!(run(input), "153");
}

// --- String node names ---

#[test]
fn string_node_names() {
    // gene1-gene2-gene3-gene4 with diagonal gene1-gene3: oracle = 7.0
    let input = "\
gene1 gene2
gene2 gene3
gene3 gene4
gene4 gene1
gene1 gene3
";
    assert_eq!(run(input), "7");
}

// --- JSON output ---

#[test]
fn json_connected() {
    let v = run_json("0 1\n1 2\n");
    assert_eq!(v["wiener_index"], serde_json::json!(4));
}

#[test]
fn json_disconnected() {
    let v = run_json("0 1\n2 3\n");
    assert_eq!(v["disconnected"], serde_json::json!(true));
    assert_eq!(v["wiener_index"], serde_json::Value::Null);
}
