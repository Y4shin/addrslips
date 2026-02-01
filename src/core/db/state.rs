use image::DynamicImage;
use sqlx::{
    Connection, Sqlite, Transaction, pool::PoolConnection, sqlite::{
        SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
    }
};
use tempdir::TempDir;
use tokio::{
    fs as async_fs,
    sync::{RwLock, RwLockReadGuard},
};

use std::{
    fs::{self, File},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};
use uuid::Uuid;
use anyhow::Context;

// NEW imports for tar + zstd
use tar::{Archive, Builder};
use zstd::stream::{read::Decoder as ZstdDecoder, write::Encoder as ZstdEncoder};

const DB_FILE_NAME: &str = "project.db";
const IMAGE_DIR_NAME: &str = "images";

pub(super) struct ProjectState {
    project_file: PathBuf,
    working_dir: TempDir,
    pool: RwLock<SqlitePool>,
}

impl std::fmt::Debug for ProjectState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectState")
            .field("project_file", &self.project_file)
            .field("working_dir", &self.working_dir.path())
            .finish()
    }
}

impl ProjectState {
    /// Acquire a pooled connection and hold the pool read lock for the entire lifetime
    /// of the returned guard.
    pub(super) async fn conn(&self) -> anyhow::Result<DbConnGuard<'_>> {
        let pool_guard = self.pool.read().await;

        // IMPORTANT: acquire the connection while the read lock is held.
        // The lock will remain held because we store it in DbConnGuard.
        let conn = pool_guard.acquire().await?;

        Ok(DbConnGuard {
            _pool_guard: pool_guard,
            conn,
        })
    }

    /// Load the image associated with the given area.
    pub(super) async fn load_area_image(
        &self,
        area_image_fname: &str,
    ) -> anyhow::Result<DynamicImage> {
        let area_img_path = self
            .working_dir
            .path()
            .join(IMAGE_DIR_NAME)
            .join(area_image_fname);
        let img = image::open(&area_img_path)
            .with_context(|| format!("Failed to open area image {:?}", area_img_path))?;
        Ok(img)
    }

    /// Save an image for the given area, returning the filename used.
    pub(super) async fn store_area_image<P: AsRef<Path>>(
        &self,
        img_path: P,
    ) -> anyhow::Result<String> {
        let images_dir = self.working_dir.path().join(IMAGE_DIR_NAME);

        let img_fname = img_path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext_str| format!("{}.{}", Uuid::new_v4(), ext_str))
            .expect("expecting extension to convert to utf-8 string");
        let dest_path = images_dir.join(&img_fname);
        async_fs::copy(&img_path, &dest_path)
            .await
            .with_context(|| format!(
                "Failed to copy area image from {:?} to {:?}",
                img_path.as_ref(),
                dest_path
            ))?;
        Ok(img_fname)
    }

    pub(super) async fn delete_area_image(&self, area_image_fname: &str) -> anyhow::Result<()> {
        let area_img_path = self
            .working_dir
            .path()
            .join(IMAGE_DIR_NAME)
            .join(area_image_fname);
        async_fs::remove_file(&area_img_path)
            .await
            .with_context(|| format!("Failed to delete area image {:?}", area_img_path))?;
        Ok(())
    }

    /// Create a tar.zst archive from the working directory.
    fn save_tar_zstd(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.project_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let out = File::create(&self.project_file)
            .with_context(|| format!("Failed to create project archive {:?}", self.project_file))?;

        // zstd encoder wrapping the output file
        let encoder = ZstdEncoder::new(out, 3)
            .with_context(|| format!("Failed to create zstd encoder for {:?}", self.project_file))?;

        // tar builder wrapping the encoder
        let mut tar = Builder::new(encoder);

        // Add entire working directory
        tar.append_dir_all(".", self.working_dir.path())
            .with_context(|| format!("Failed to add {:?} to tar", self.working_dir.path()))?;

        // Finish tar, then finish zstd stream
        let encoder = tar.into_inner()
            .with_context(|| format!("Failed to finalize tar for {:?}", self.project_file))?;

        encoder.finish()
            .with_context(|| format!("Failed to finalize zstd stream for {:?}", self.project_file))?;

        Ok(())
    }

    /// Exclusive close+pack:
    /// - waits for all in-flight read queries (because it takes a WRITE lock)
    /// - checkpoints WAL to ensure project.db is current
    /// - closes pool to release file handles
    /// - archives working dir
    pub(super) async fn save_project(&self) -> anyhow::Result<()> {
        self.internal_close_and_pack(true).await
    }

    pub(super) async fn internal_close_and_pack(&self, reopen: bool) -> anyhow::Result<()> {
        // Take exclusive write lock for the whole operation:
        // this guarantees no queries run while we checkpoint/close/pack.
        let mut pool_guard = self.pool.write().await;

        // Flush WAL into main DB and truncate it
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE);")
            .execute(&*pool_guard)
            .await?;

        // Release file handles (important on Windows); this is "final".
        // After this, any DB use will fail unless you re-open a new pool.
        pool_guard.close().await;

        // Now pack files (db file is stable and handles should be released).
        // Note: this is synchronous IO; consider spawn_blocking for large projects.
        self.save_tar_zstd()?;

        // Now re-open the pool for any future use.
        if reopen {
            let db_file = self.working_dir.path().join(DB_FILE_NAME);
            let connect_opts = SqliteConnectOptions::new()
                .filename(&db_file)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .foreign_keys(true);

            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(connect_opts)
                .await?;
            *pool_guard = pool;
        }
        Ok(())
    }

    pub(super) async fn new<P: AsRef<Path>>(project_file: P) -> anyhow::Result<Self> {
        let project_file = project_file.as_ref().to_path_buf();

        // Ensure project file exists; if not, create an empty tar.zst at that location (if parent exists).
        if !project_file.is_file() {
            if project_file.parent().map(|p| p.is_dir()).unwrap_or(false) {
                let out = File::create(&project_file)
                    .with_context(|| format!("Failed to create project archive {:?}", project_file))?;

                let encoder = ZstdEncoder::new(out, 3)
                    .with_context(|| format!("Failed to create zstd encoder for {:?}", project_file))?;

                let tar = Builder::new(encoder);
                let encoder = tar.into_inner()
                    .with_context(|| format!("Failed to finalize empty tar {:?}", project_file))?;

                encoder.finish()
                    .with_context(|| format!("Failed to finalize empty zstd stream {:?}", project_file))?;
            } else {
                anyhow::bail!("Project file parent does not exist: {:?}", project_file);
            }
        }

        // Create working directory
        let working_dir = TempDir::new("addrslips_project")?;

        // Unpack tar.zst project file into working dir.
        {
            let f = File::open(&project_file)
                .with_context(|| format!("Failed to open project archive {:?}", project_file))?;

            let decoder = ZstdDecoder::new(f)
                .with_context(|| format!("Invalid zstd stream in {:?}", project_file))?;

            let mut archive = Archive::new(decoder);
            archive.unpack(working_dir.path())
                .with_context(|| format!(
                    "Failed to extract archive {:?} into {:?}",
                    project_file,
                    working_dir.path()
                ))?;
        }

        // Project layout expectations
        let db_file = working_dir.path().join(DB_FILE_NAME);
        let images_dir = working_dir.path().join(IMAGE_DIR_NAME);

        let db_exists = db_file.is_file();
        let images_exist = images_dir.is_dir();

        match (db_exists, images_exist) {
            (true, true) => {}
            (false, false) => {
                fs::create_dir_all(&images_dir)?;
                File::create(&db_file)?;
            }
            (true, false) => anyhow::bail!(
                "Corrupt project: database exists ({:?}) but images dir missing ({:?})",
                db_file,
                images_dir
            ),
            (false, true) => anyhow::bail!(
                "Corrupt project: images dir exists ({:?}) but database missing ({:?})",
                images_dir,
                db_file
            ),
        }

        let connect_opts = SqliteConnectOptions::new()
            .filename(&db_file)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connect_opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self {
            project_file,
            working_dir,
            pool: RwLock::new(pool),
        })
    }
}

pub struct DbConnGuard<'a> {
    _pool_guard: RwLockReadGuard<'a, SqlitePool>,
    conn: PoolConnection<Sqlite>,
}

impl<'a> Deref for DbConnGuard<'a> {
    type Target = PoolConnection<Sqlite>;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl<'a> DerefMut for DbConnGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

impl Drop for ProjectState {
    fn drop(&mut self) {
        // Try to save using existing runtime, fall back to creating one if needed
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // We're in a Tokio runtime context, but we can't block_on from within
            // a runtime. Spawn a blocking task instead.
            std::thread::spawn(move || {
                // This won't work either - we need to just skip save-on-drop in async context
                // and rely on explicit save() calls
            });
            // For now, skip save when already in async context
            // Users should call save_project() explicitly before dropping
            Ok(())
        } else {
            // No runtime available, create a temporary one for cleanup
            // This is heavyweight but ensures save-on-drop semantics are preserved
            match tokio::runtime::Runtime::new() {
                Ok(rt) => rt.block_on(async { self.internal_close_and_pack(false).await }),
                Err(e) => Err(e.into()),
            }
        };

        // Log errors but don't panic in Drop
        if let Err(e) = result {
            eprintln!("Warning: Failed to save project on drop: {}", e);
        }
    }
}

impl<'a> DbConnGuard<'a> {
    pub(super) async fn begin_transaction(&'a mut self) -> anyhow::Result<Transaction<'a, Sqlite>> {
        Ok(self.conn.begin().await?)
    }
}