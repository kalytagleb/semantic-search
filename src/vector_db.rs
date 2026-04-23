use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, params};
use std::path::Path;

pub struct VectorDb {
    conn: Connection,
}

pub struct VectorRecord {
    pub path: String,
    pub chunk_index: usize,
    pub chunk_text: String,
    pub vector: Vec<f32>,
    pub content_hash: String,
}

impl VectorDb {
    pub fn open(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create data directory")?;
        }
        let conn = Connection::open(db_path).context("Failed to open SQLite database")?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .context("Failed to set SQLite pragmas")?;
        let db = VectorDb { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS meta (
                    key   TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS chunks (
                    id           INTEGER PRIMARY KEY AUTOINCREMENT,
                    path         TEXT NOT NULL,
                    content_hash TEXT NOT NULL,
                    chunk_index  INTEGER NOT NULL,
                    chunk_text   TEXT NOT NULL,
                    vector       BLOB NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_chunks_path ON chunks(path);",
            )
            .context("Failed to initialize database schema")?;
        Ok(())
    }

    pub fn get_model_id(&self) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM meta WHERE key = 'model_id'",
            [],
            |row| row.get(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn set_model_id(&self, model_id: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('model_id', ?1)",
                params![model_id],
            )
            .context("Failed to set model ID")?;
        Ok(())
    }

    pub fn clear_chunks(&self) -> Result<()> {
        self.conn
            .execute("DELETE FROM chunks", [])
            .context("Failed to clear chunks")?;
        Ok(())
    }

    pub fn file_needs_reindex(&self, path: &str, new_hash: &str) -> Result<bool> {
        let result: rusqlite::Result<String> = self.conn.query_row(
            "SELECT content_hash FROM chunks WHERE path = ?1 LIMIT 1",
            params![path],
            |row| row.get(0),
        );
        match result {
            Ok(stored) => Ok(stored != new_hash),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(true),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn delete_file_chunks(&self, path: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM chunks WHERE path = ?1", params![path])
            .context("Failed to delete file chunks")?;
        Ok(())
    }

    pub fn insert_chunk(&self, record: &VectorRecord) -> Result<()> {
        let vector_bytes = vector_to_bytes(&record.vector);
        self.conn
            .execute(
                "INSERT INTO chunks (path, content_hash, chunk_index, chunk_text, vector)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    record.path,
                    record.content_hash,
                    record.chunk_index as i64,
                    record.chunk_text,
                    vector_bytes,
                ],
            )
            .context("Failed to insert chunk")?;
        Ok(())
    }

    pub fn load_all_vectors(&self) -> Result<Vec<VectorRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT path, content_hash, chunk_index, chunk_text, vector FROM chunks",
            )
            .context("Failed to prepare query")?;

        let records: Vec<VectorRecord> = stmt
            .query_map([], |row| {
                let vector_bytes: Vec<u8> = row.get(4)?;
                Ok(VectorRecord {
                    path: row.get(0)?,
                    content_hash: row.get(1)?,
                    chunk_index: row.get::<_, i64>(2)? as usize,
                    chunk_text: row.get(3)?,
                    vector: bytes_to_vector(&vector_bytes),
                })
            })
            .context("Failed to query chunks")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(records)
    }

    pub fn chunk_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
            .context("Failed to count chunks")
    }
}

fn vector_to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn bytes_to_vector(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db() -> (VectorDb, PathBuf) {
        let path = std::env::temp_dir().join("test_semantic_search_vectors.db");
        if path.exists() {
            std::fs::remove_file(&path).ok();
        }
        let db = VectorDb::open(&path).expect("Failed to open test DB");
        (db, path)
    }

    fn make_record(path: &str, idx: usize, text: &str) -> VectorRecord {
        VectorRecord {
            path: path.to_string(),
            chunk_index: idx,
            chunk_text: text.to_string(),
            vector: vec![0.1, 0.2, 0.3],
            content_hash: "abc123".to_string(),
        }
    }

    #[test]
    fn test_open_creates_schema() {
        let (db, path) = temp_db();
        assert_eq!(db.chunk_count().unwrap(), 0);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_insert_and_load() {
        let (db, path) = temp_db();

        db.insert_chunk(&make_record("file.txt", 0, "hello world")).unwrap();
        db.insert_chunk(&make_record("file.txt", 1, "привет мир")).unwrap();

        let records = db.load_all_vectors().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].path, "file.txt");

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_incremental_indexing_skips_unchanged() {
        let (db, path) = temp_db();

        db.insert_chunk(&make_record("notes.md", 0, "some content")).unwrap();

        assert!(!db.file_needs_reindex("notes.md", "abc123").unwrap());
        assert!(db.file_needs_reindex("notes.md", "different_hash").unwrap());
        assert!(db.file_needs_reindex("new_file.md", "any_hash").unwrap());

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_model_id_stored_and_retrieved() {
        let (db, path) = temp_db();

        assert!(db.get_model_id().unwrap().is_none());
        db.set_model_id("EmbeddingGemma300M").unwrap();
        assert_eq!(db.get_model_id().unwrap().unwrap(), "EmbeddingGemma300M");

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_large_vector_round_trip() {
        let (db, path) = temp_db();

        let big_vector: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
        let mut record = make_record("large.bin", 0, "test");
        record.vector = big_vector.clone();

        db.insert_chunk(&record).unwrap();
        let loaded = db.load_all_vectors().unwrap();

        assert_eq!(loaded[0].vector.len(), 384);
        for (a, b) in loaded[0].vector.iter().zip(big_vector.iter()) {
            assert!((a - b).abs() < 1e-6);
        }

        std::fs::remove_file(path).ok();
    }
}
