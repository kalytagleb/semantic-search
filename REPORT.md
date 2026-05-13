# Project Report: Local Semantic Search Engine

**Team:** Iaroslav Golov, Hlib Kalyta, Yaroslav Shyshmylo

---

## 1. Introduction

The idea came from a simple frustration: modern file managers and OS search tools only do keyword matching. If you remember roughly what a file was about but not its name or exact wording, you are stuck. We wanted to build something that works the way memory actually works — you describe the concept, and the tool figures out which file matches.

The result is a local semantic search engine written in Rust. You point it at a directory, it indexes all the readable files, and from that point you can search by describing what you are looking for in plain English or Russian. The key constraint we set early on was that everything had to run completely offline — no cloud APIs, no sending data anywhere. The embedding model runs on your machine, the indexes live on your disk.

The final application has both a command-line interface and a terminal UI, supports plain text, Markdown, source code, PDF and DOCX files, and is smart enough to skip files that have not changed when you re-run the indexer.

---

## 2. Requirements

The core requirement was to take a natural language query and return the most relevant files from a local directory tree. To make that work, the system needed to scan files recursively, extract their text content regardless of format, split that content into chunks small enough for the AI model to process, generate a vector embedding per chunk, and store everything in a searchable index.

We also wanted a second, complementary index based on keyword matching (BM25) and a way to combine both into a single ranked result list. That combination is what makes the search useful in practice — pure semantic search is weak on exact names and abbreviations, pure keyword search misses meaning entirely.

On top of the search itself, the index needed to be incremental. Re-indexing a large directory every time you add one file is not acceptable, so unchanged files had to be detected and skipped automatically.

The TUI was originally a stretch goal but we ended up implementing it as the default interface. Filtering by file extension and modification date were added later once the core search was working.

---

## 3. Design

The application is split into a pipeline of loosely coupled modules. The scanner walks the directory tree using the `ignore` crate, which respects `.gitignore` files and skips hidden directories. Each file gets checked against a set of filters — size limit of 50 MB, binary detection via magic bytes — before being passed to the extractor.

The extractor reads file content in a format-aware way. PDFs go through `pdf-extract`, DOCX files through `docx-rs`, and everything else is read as UTF-8 with lossy conversion. File type is determined by reading the first 16 bytes and checking known binary signatures rather than trusting the extension, because a renamed file would otherwise cause a crash.

The chunker then splits the text into overlapping segments. Code files use smaller chunks of around 600 characters with 150-character overlap, prose text and documents use 800 with 200 overlap. The overlap ensures that sentences sitting on a chunk boundary are not lost.

Each chunk gets embedded by the AI engine, which wraps the Gemma 300M model through `fastembed`. Chunks are indexed as passages with a specific prefix format that the model was trained to expect. Queries at search time use a different prefix. This asymmetry matters — without it, the cosine similarity between a query and a document is noticeably worse because the model maps them into different parts of the vector space.

The vectors are stored as raw binary blobs in SQLite alongside a SHA-256 hash of the file content. The hash is what drives incremental indexing: on each run, if the hash matches what is already stored, the file is skipped. If it changed, the old chunks are deleted and the file is reprocessed.

In parallel, the same chunks go into a Tantivy BM25 index for keyword search. When a query comes in, both indexes are searched and the results are merged using Reciprocal Rank Fusion. The formula adds `1 / (60 + rank)` for each list, so a file that appears high in both gets a much better combined score than one that dominates only one of them.

For the TUI, the backend runs on a separate thread and communicates with the frontend through typed `mpsc` channels. The backend sends progress events during indexing and search results when a query completes. The frontend just renders whatever it receives, which keeps it responsive even when the AI model is loading.

---

## 4. Design Choices

The most significant early decision was how to store vectors. Our first version used a single binary flat file — we serialized all vectors into it and loaded the whole thing on every search. That worked until we tried to add incremental indexing, at which point it became clear that rewriting the entire file to update one entry was not going to work. There was also no crash safety. We moved to SQLite, which gave us row-level updates, atomic writes, and WAL mode for concurrent reads, all without adding any server process or external dependency.

We also considered dedicated vector databases like Qdrant. They support approximate nearest neighbor search, which is faster at scale, but they run as separate services and require installation. That goes against the local-first principle of the project, where the whole point is that the tool works anywhere without setup.

The decision to do brute-force cosine similarity instead of an ANN index was deliberate. For a personal machine the real bottleneck during search is loading the model and generating the query embedding, not the vector comparison itself. Running `par_iter()` from `rayon` across all stored vectors takes a fraction of a second on typical hardware, so the extra complexity of HNSW was not worth it.

For file type detection we could have trusted file extensions, but that creates a failure case where a binary file with a `.txt` extension gets passed to the text parser and produces garbage or a panic. Reading magic bytes first is slightly more expensive but eliminates that class of bug entirely.

---

## 5. Dependencies

`fastembed` provides the Gemma 300M embedding model and handles downloading and caching it on first use. `tantivy` is the BM25 full-text search engine. `rusqlite` is the SQLite driver used for vector and metadata storage. `text-splitter` handles the chunking with configurable size and overlap. `rayon` parallelizes the cosine similarity computation across all stored vectors.

`ignore` does directory traversal with `.gitignore` support. `infer` identifies file types from magic bytes. `pdf-extract` and `docx-rs` extract text from binary document formats. `clap` handles command-line argument parsing. `ratatui` and `crossterm` together make the terminal UI work cross-platform. `sha2` computes content hashes for incremental indexing. `chrono` parses dates for the `--since` filter. `directories` resolves the user data directory on each OS. `anyhow` is used throughout for error handling.

---

## 6. Evaluation

Overall we are satisfied with how the project turned out. The hybrid search works well and the TUI makes the tool feel like a real application rather than a prototype. The incremental indexing is reliable — on a second run over the same directory almost everything is skipped instantly.

The vector storage migration was the biggest setback. We lost a couple of days redesigning the storage layer after the flat-file approach hit its limits. In hindsight we should have gone with SQLite from the start, but at the time a simple file felt easier to reason about. The lesson is that incremental updates need to be a first-class concern from day one, not retrofitted later.

Search quality surprised us in a different way than expected. The semantic model is good at understanding meaning but it assumes the query describes the content of a file. If you write something like "find that game I wrote last week," the model does not know what game you mean. You have to write "python script implementing snake with keyboard controls" to get a useful result. That is just how embedding models work, but we did not fully appreciate it until we actually used the tool on our own files. It changes how you interact with the search — it is less like a chat assistant and more like a very smart content-based filter.

Implementing a project of this scale in Rust was harder than the same thing would be in Python, but in a way that felt productive. The borrow checker forced us to structure the code more carefully — for example, passing a mutable reference to the AI engine through the indexing pipeline while also holding a reference to the database required some thought. In Python you just pass objects around and think about it later. In Rust you think about it now.

What we got in return was confidence. Once the code compiled cleanly, whole categories of bugs simply did not exist — no data races between the TUI thread and the backend, no use-after-free, no null pointer issues. The `rayon` parallelism in particular would have required careful locking in most other languages; here it was a one-line change that the compiler verified was safe.

Compile times were genuinely painful. A clean build with `fastembed` and `tantivy` takes several minutes. Changing a single function in the search module and waiting three minutes to test it gets old quickly. Incremental compilation helped, but cold builds during CI or on a fresh checkout were slow enough to be noticeable.
