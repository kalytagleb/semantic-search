# Local Semantic Search (PROPOSAL)

## Team Members
* Iaroslav Golov
* Hlib Kalyta
* Yaroslav Shyshmylo

## Introduction
Modern operating systems lack the ability to intuitively search for local files based on meaning or concepts rather than exact keyword matches. Our project solves this problem by building a completely local, privacy-first CLI application (with a planned TUI) in Rust that performs Hybrid Semantic Search. 

**What we hope to learn:** We want to learn how to interact with the file system efficiently, integrate Machine Learning models (embeddings) natively in Rust without relying on Python, and build a high-performance local search engine.

## Requirements in brief
For the project to be deemed successful, it must:
* Recursively scan local directories and parse text from various file formats (txt, md, pdf, rs, py, etc.), ignoring system/build directories (e.g., node_modules, .git).
* Intelligently chunk the extracted text so accuracy isn't lost on large files.
* Build an Inverted Index (for exact keyword match / BM25).
* Generate vector embeddings locally using a small AI model and store them in a Vector Index.
* Allow users to query the application via CLI and return a combined list of relevant files using Reciprocal Rank Fusion (RRF).

## Dependencies
We plan to use the following crates from lib.rs:
* **walkdir** - for efficient filesystem traversal.
* **text-splitter** - for intelligent text chunking.
* **tantivy** - for the exact-match inverted index.
* **ort** or **candle-core** - for local neural network inference (generating embeddings).
* **clap** - for command-line argument parsing.
* **ratatui** (optional/stretch goal) - for the terminal user interface.

## Architecture & Workflow (Rudimentary Diagram)

Below is the architecture showing how our components interact when the user runs the index and search commands.

    [ File System ]
          |
          V
     1. Scanner    ---(Finds files, filters extensions, watches changes)
          |
          V
     2. Extractor  ---(Reads files, breaks text into ~200 word chunks)
          |
          V
     3. Indexer    ---(Processes chunks in parallel)
          |--> Inverted Index (BM25 for keywords via Tantivy)
          |--> Vector Index (Embeddings for semantic search via Ort/Candle)
          |
          V
     5. Storage    ---(Saves both indexes to HDD)
          |
          V
     4. Searcher   ---(Accepts query, searches both indexes, joins via RRF)
          ^
          |
     6. CLI / TUI  ---(User input/output: formats and displays results)

## How it looks in practice (Usage Example)

**First launch (Indexing)**

    $ semantic-search index ~/Documents
        .. File scanning
        Find: 2300 files with indicated extensions
        Indexed: <progress-bar> 450/2300
        Remaining: ~23 minutes

**Search**

    $ semantic-search search "memory management without Garbage Collector"
         1) ~/Documents/rust_book.md
             "Ownership allows us manage memory..."
         2) ~/Documents/system_programming.pdf
             "...In contrast, in Java there use Garbage Collector..."