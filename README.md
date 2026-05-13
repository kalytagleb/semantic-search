[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/RHGu4AQi)
# Semantic Search Engine

A local semantic search tool that finds files by meaning, not just keywords. Built in Rust.

**Example:** you type `"article about memory management"` — it finds `rust_book.md` even if those exact words aren't in the filename.

## How it works

Two search engines run in parallel and their results are merged:
- **Semantic (vector)** — understands meaning via an embedding model (Gemma 300M)
- **Keyword (BM25)** — fast exact word matching via Tantivy

Results are ranked by combining both scores using Reciprocal Rank Fusion.

## Build

```bash
cargo build --release
```

On first run the embedding model (~300 MB) downloads automatically and gets cached locally.

## Usage

### Index files

```bash
# Index a specific folder
cargo run -- index ~/Documents

# Index everything (takes a while on first run, fast on repeat)
cargo run -- index /

# Index current directory
cargo run -- index .
```

Re-indexing is incremental — files that haven't changed are skipped automatically.

### Search

```bash
cargo run -- search "your query here"
```

Returns top 10 results with file path and a short text preview.

## Writing good queries

Write queries in **natural language** — describe what you're looking for, not just keywords.

| Query | What it finds |
|---|---|
| `"snake game python"` | Python script implementing snake |
| `"how to handle errors in rust"` | Rust files about error handling |
| `"university lecture notes math"` | Notes, PDFs with math content |
| `"config file for database connection"` | YAML/TOML configs with DB settings |
| `"flying bird avoid obstacles"` | Flappy Bird style game code |

**Tips:**
- Longer, descriptive queries work better than single words
- Mix content description with file type: `"python script that sorts a list"`
- Both English and Russian queries work

## Index location

The index is stored at `~/.local/share/semantic-search/` (Linux) and does not affect your files in any way.

## Architecture

```
Scanner   →  finds files, skips binaries and system dirs
Extractor →  reads text, splits into ~600 char chunks
AI Engine →  generates embedding vector per chunk (Gemma 300M)
Vector DB →  stores vectors in SQLite, supports incremental updates
Tantivy   →  BM25 full-text index
Searcher  →  queries both indexes, merges with RRF, returns top 10
```
