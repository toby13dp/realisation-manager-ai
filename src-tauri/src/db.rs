//! SQLite connection pool + migrations.
//!
//! We use r2d2 to maintain a pool of rusqlite connections. The pool is
//! wrapped in a thin `DbPool` newtype so the rest of the crate can share it
//! easily across threads.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use parking_lot::Mutex;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use refinery::embed_migrations;

embed_migrations!("src-tauri/migrations");

pub type DbPoolInner = Pool<SqliteConnectionManager>;

/// Wraps an r2d2 pool. Cloning is cheap — it just clones the inner `Arc`.
#[derive(Clone)]
pub struct DbPool {
    inner: DbPoolInner,
    /// Single-writer mutex. SQLite handles concurrent reads fine via WAL,
    /// but we serialise writes to avoid `database is locked` errors.
    write_lock: Arc<Mutex<()>>,
}

impl DbPool {
    pub fn get(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.inner.get().context("db pool get")
    }

    /// Acquire the write lock and a connection. Use for any mutating query.
    pub fn write(&self) -> Result<WriteGuard<'_>> {
        let guard = self.write_lock.lock();
        let conn = self.inner.get().context("db pool get (write)")?;
        Ok(WriteGuard { _lock: guard, conn })
    }

    pub fn write_lock(&self) -> &Arc<Mutex<()>> {
        &self.write_lock
    }
}

pub struct WriteGuard<'a> {
    _lock: parking_lot::MutexGuard<'a, ()>,
    pub conn: r2d2::PooledConnection<SqliteConnectionManager>,
}

/// Open the SQLite database at `path`, configure WAL mode, run migrations.
pub fn open_and_migrate(path: &Path) -> Result<DbPool> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create db dir {}", parent.display()))?;
    }

    let manager = SqliteConnectionManager::file(path)
        .with_init(|c| {
            c.execute_batch(
                "PRAGMA journal_mode = WAL; \
                 PRAGMA synchronous = NORMAL; \
                 PRAGMA foreign_keys = ON; \
                 PRAGMA temp_store = MEMORY; \
                 PRAGMA mmap_size = 268435456;",
            )
        });

    let pool = Pool::builder()
        .max_size(8)
        .min_idle(Some(2))
        .connection_timeout(Duration::from_secs(10))
        .connection_customizer(Box::new(SqliteCustomizer))
        .build(manager)
        .context("build r2d2 pool")?;

    // Run migrations on a dedicated connection.
    {
        let mut conn = pool.get().context("acquire migration conn")?;
        migrations::runner()
            .run(&mut *conn)
            .context("run refinery migrations")?;
    }

    let write_lock = Arc::new(Mutex::new(()));
    Ok(DbPool { inner: pool, write_lock })
}

#[derive(Debug)]
struct SqliteCustomizer;

impl r2d2::CustomizeConnection<rusqlite::Connection, rusqlite::Error> for SqliteCustomizer {
    fn on_acquire(&self, conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "PRAGMA foreign_keys = ON; \
             PRAGMA busy_timeout = 5000;",
        )?;
        Ok(())
    }
}
