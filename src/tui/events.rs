use crate::vector_db::SearchResult;

pub enum BackendEvent {
    Progress(f64, String),
    IndexingFinished,
    SearchResults(Vec<SearchResult>),
    Error(String),
}

pub enum FrontendEvent {
    Search(String),
    Quit,
}
