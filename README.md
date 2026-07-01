# rsomics-wiener-index

Wiener index of an undirected graph: sum of shortest-path distances over all unordered node pairs.

Value-exact port of `networkx.wiener_index` (Python, BSD-3-Clause).

## Usage

```
echo "0 1
1 2
2 3" | rsomics-wiener-index
# → 10

echo "0 1
2 3" | rsomics-wiener-index
# → inf
```

Input: one edge per line (`u v`), `#`-comments and blank lines skipped.  
Parallel edges are deduplicated (matching `nx.Graph` behaviour).  
Disconnected graph → prints `inf` (matching networkx).

```
rsomics-wiener-index --json
# → {"wiener_index": 10}
# → {"wiener_index": null, "disconnected": true}   (for disconnected)
```

## Performance

600-node, 3000-edge connected graph (gnm seed=7):

| Tool | Mean wall time |
|------|---------------|
| networkx 3.6.1 (Python) | 180 ms |
| rsomics-wiener-index 0.1.0 | 7.2 ms |
| **ratio** | **25×** |

Machine: aarch64-apple-darwin (Apple M2), single thread.  
Algorithm: integer-indexed adjacency list + BFS from every node.

## Origin

This crate is an independent Rust reimplementation of `networkx.wiener_index` based on:
- The NetworkX source (BSD-3-Clause): `networkx/algorithms/wiener.py`
- The definition: W = Σ_{unordered pairs {u,v}} d(u,v), where d is unweighted BFS distance.
  For disconnected graphs, W = ∞ (any unreachable pair contributes ∞).

NetworkX is BSD-3-Clause; its source was read for algorithm correctness (not GPL; clean-room
rules do not apply). Test oracle values were produced with networkx 3.6.1.

License: MIT OR Apache-2.0.  
Upstream credit: NetworkX <https://networkx.org> (BSD-3-Clause).
