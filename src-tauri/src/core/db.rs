use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tracing::info;

/// Thread-safe database path holder.
/// Each command opens its own connection from the path, avoiding `Connection`'s
/// lack of `Send + Sync`.
#[derive(Clone)]
pub struct Database {
    pub db_path: Arc<PathBuf>,
}

impl Database {
    const PRAGMA_CONFIG: &'static str = "
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;
        PRAGMA busy_timeout=5000;
    ";

    fn configure_connection(conn: &Connection) -> Result<()> {
        conn.execute_batch(Self::PRAGMA_CONFIG)?;
        Ok(())
    }

    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let db_path = get_db_path(app_handle);
        info!("Database path: {:?}", db_path);

        let conn = Connection::open(&db_path)?;
        Self::configure_connection(&conn)?;

        Ok(Database { db_path: Arc::new(db_path) })
    }

    /// Test-friendly constructor that takes a direct path string.
    /// Only available in test builds.
    #[cfg(test)]
    pub fn new_from_path(path: &str) -> Result<Self> {
        let db_path = PathBuf::from(path);
        let conn = Connection::open(&db_path)?;
        Self::configure_connection(&conn)?;
        Ok(Database { db_path: Arc::new(db_path) })
    }

    pub fn open_connection(&self) -> Result<Connection> {
        let conn = Connection::open(&*self.db_path)?;
        Self::configure_connection(&conn)?;
        Ok(conn)
    }

    /// Convenience method that runs migrations. Alias for run_migrations().
    pub fn init(&self) -> Result<()> {
        self.run_migrations()
    }

    pub fn run_migrations(&self) -> Result<()> {
        let conn = self.open_connection()?;
        info!("Running database migrations...");

        // Get current version
        let user_version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

        if user_version < 1 {
            info!("Applying migration v1: initial schema");
            drop(conn);
            self.apply_v1_initial_schema()?;
        } else {
            info!("Database is up to date (version {})", user_version);
        }

        Ok(())
    }
    
    fn apply_v1_initial_schema(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT UNIQUE NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                file_hash TEXT,
                mime_type TEXT,
                width INTEGER,
                height INTEGER,
                thumbnail_path TEXT,
                phash TEXT,
                exif_data JSON,
                ai_status TEXT NOT NULL DEFAULT 'pending',
                ai_tags JSON,
                ai_description TEXT,
                ai_category TEXT,
                ai_confidence REAL,
                ai_model TEXT,
                ai_processed_at DATETIME,
                ai_error_message TEXT,
                ai_retry_count INTEGER NOT NULL DEFAULT 0,
                source TEXT NOT NULL DEFAULT 'import',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_images_ai_status ON images(ai_status);
            CREATE INDEX IF NOT EXISTS idx_images_created_at ON images(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_images_file_hash ON images(file_hash);
            
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL COLLATE NOCASE,
                count INTEGER NOT NULL DEFAULT 0
            );
            
            CREATE TABLE IF NOT EXISTS image_tags (
                image_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (image_id, tag_id),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );
            
            CREATE TABLE IF NOT EXISTS search_index (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                term TEXT NOT NULL,
                image_id INTEGER NOT NULL,
                field TEXT NOT NULL,
                position INTEGER,
                weight REAL NOT NULL DEFAULT 1.0,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_search_index_term ON search_index(term);
            CREATE INDEX IF NOT EXISTS idx_search_index_image_id ON search_index(image_id);
            
            CREATE TABLE IF NOT EXISTS task_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                image_id INTEGER NOT NULL,
                task_type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                priority INTEGER NOT NULL DEFAULT 0,
                retry_count INTEGER NOT NULL DEFAULT 0,
                max_retries INTEGER NOT NULL DEFAULT 3,
                error_message TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                started_at DATETIME,
                completed_at DATETIME,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_task_queue_status ON task_queue(status);
            CREATE INDEX IF NOT EXISTS idx_task_queue_priority ON task_queue(priority DESC, created_at ASC);
            
            CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            
            INSERT OR IGNORE INTO app_config (key, value) VALUES
                ('lm_studio_url', 'http://localhost:1234'),
                ('ai_concurrency', '3'),
                ('ai_timeout_seconds', '60'),
                ('ai_max_retries', '3'),
                ('theme', 'system'),
                ('language', 'zh-CN'),
                ('thumbnail_size', '300');
            
            PRAGMA user_version = 1;
        ")?;
        
        Ok(())
    }
}

fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let app_data = app_handle.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap()
    });
    
    std::fs::create_dir_all(&app_data).unwrap();
    app_data.join("arcanecodex.db")
}

pub fn init_database(app_handle: &tauri::AppHandle) -> Result<()> {
    let db = Database::new(app_handle)?;
    db.run_migrations()?;
    info!("Database initialized successfully");
    Ok(())
}
