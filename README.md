We should build console app (TUI) in Rust, which index all files on computer and allow us to find it by essential language (english, russian, etc.). 
For example, you input: "article about memory management" - it finds all files, even if file name does not contain that words.
How doest it should look in practice? 
# First launch
$ semantic-search index ~/Documents
    .. File scanning
    Find: 2300 files with indicated extensions
    Indexed: <progress-bar> 450/2300
    Remaining: ~23 minutes
# Search
$ semantic-search search "memory management without Garbage Collector"
     1) ~/Documents/rust_book.md
         "Ownership allows us manage memory"
     2) ~/Documents/system_programming.pdf
         "...In contrast, in Java there use Garbage Collector"
# Interactive mode
$ semantic-search tui

Component 1: Scanner
It should go inside all file system, beginning from indicated folder. It returns a list of files, which we should index. It follows by the changes with file watcher. 
Supported extension: "txt" | "md" | "pdf" | "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "java" |"yaml" | "json" | "toml" | "csv" | "docx" | "html"
This component only finds and follows.

Component 2: Extractor
As argument it gets path to file. It returns text from files. 
# Input: ~/Documents/rust_book.md
# Output: content: String - all text in file
          chunks: Vec<String> - text divided into chunks ~200 words
          metadata: Metadata
Why should we use broke all on chunks? 
For example, there are file on 100 pages. If we create 1 embedding for all file - loses accuracy. We should divide it on chunks a create embedding for every chunks. 
This component only read file and return text.

Component 3: Indexer
It receives text from Extractor. It builds two indexes parallely:
Inverted index - for search by key words (fast, accurate)
Vector index - for semantic search (clever, understand sense)
It should save data in Storage (HDD)
    Inverted index
        Word -> file list, where it is
        There use algorithm BM25.
    Vector Index
        Every chunk -> embedding (vector from 384 numbers)

        Model: I don't know (you can write your recommendations)

Component 4: Searcher
It accepts text query. Find in both indexes. Join results. 
Input: text query
Output: Vec<SearchResult> - list of result in the descending order (depends from relevancy)

Component 5: Storage
All indexes should save in reboot.

Componet 6: TUI
