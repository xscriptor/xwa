use std::path::Path;

use rusqlite::Connection;
use serde_json::Value;
use tracing::info;

#[derive(Clone)]
pub struct DbStore {
    conn: std::sync::Arc<std::sync::Mutex<Connection>>,
}

impl DbStore {
    pub fn new(path: &str) -> Result<Self, String> {
        let exists = Path::new(path).exists();
        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open DB: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'queued',
                created_at TEXT NOT NULL,
                pages_scraped INTEGER DEFAULT 0,
                files_downloaded INTEGER DEFAULT 0,
                total_pages INTEGER DEFAULT 0,
                current_url TEXT,
                errors TEXT DEFAULT '[]',
                emails TEXT DEFAULT '[]',
                phones TEXT DEFAULT '[]',
                data TEXT
            );
            CREATE TABLE IF NOT EXISTS deep_results (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL DEFAULT '',
                url TEXT NOT NULL,
                structured_data TEXT,
                nlp_data TEXT,
                emails TEXT DEFAULT '[]',
                phones TEXT DEFAULT '[]',
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS schedules (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                interval_min INTEGER NOT NULL DEFAULT 60,
                config TEXT NOT NULL DEFAULT '{}',
                enabled INTEGER NOT NULL DEFAULT 1,
                last_run TEXT,
                next_run TEXT NOT NULL,
                created_at TEXT NOT NULL
            );"
        ).map_err(|e| format!("Failed to create table: {}", e))?;

        if !exists {
            info!("Created new database at {}", path);
        } else {
            info!("Loaded database from {}", path);
        }

        Ok(Self { conn: std::sync::Arc::new(std::sync::Mutex::new(conn)) })
    }

    pub fn save_job(&self, job: &crate::api::routes::JobInfo) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO jobs (id, url, status, created_at, pages_scraped, files_downloaded, total_pages, current_url, errors, emails, phones)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                job.id, job.url, job.status, job.created_at,
                job.pages_scraped as i64, job.files_downloaded as i64, job.total_pages as i64,
                job.current_url,
                serde_json::to_string(&job.errors).unwrap_or_default(),
                serde_json::to_string(&job.emails).unwrap_or_default(),
                serde_json::to_string(&job.phones).unwrap_or_default(),
            ],
        ).map_err(|e| format!("Failed to save job: {}", e))?;
        Ok(())
    }

    pub fn load_jobs(&self) -> Result<Vec<crate::api::routes::JobInfo>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, url, status, created_at, pages_scraped, files_downloaded, total_pages, current_url, errors, emails, phones FROM jobs ORDER BY created_at DESC"
        ).map_err(|e| format!("Failed to prepare: {}", e))?;

        let jobs = stmt.query_map([], |row| {
            let errors_str: String = row.get(8)?;
            let emails_str: String = row.get(9)?;
            let phones_str: String = row.get(10)?;

            Ok(crate::api::routes::JobInfo {
                id: row.get(0)?,
                url: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
                pages_scraped: row.get::<_, i64>(4)? as usize,
                files_downloaded: row.get::<_, i64>(5)? as usize,
                total_pages: row.get::<_, i64>(6)? as usize,
                current_url: row.get(7)?,
                errors: serde_json::from_str(&errors_str).unwrap_or_default(),
                emails: serde_json::from_str(&emails_str).unwrap_or_default(),
                phones: serde_json::from_str(&phones_str).unwrap_or_default(),
            })
        }).map_err(|e| format!("Failed to query: {}", e))?;

        let mut result = Vec::new();
        for job in jobs {
            result.push(job.map_err(|e| format!("Failed to read row: {}", e))?);
        }
        Ok(result)
    }

    pub fn export_all(&self) -> Result<Value, String> {
        let jobs = self.load_jobs()?;
        Ok(serde_json::json!({
            "version": "SHINOBI_DB_V1",
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "jobs": jobs,
        }))
    }

    pub fn import_jobs(&self, jobs: &[crate::api::routes::JobInfo]) -> Result<usize, String> {
        let mut count = 0;
        for job in jobs {
            self.save_job(job)?;
            count += 1;
        }
        Ok(count)
    }

    pub fn save_deep_result(&self, result: &crate::api::routes::DeepResult) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO deep_results (id, job_id, url, structured_data, nlp_data, emails, phones, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                result.id,
                result.job_id,
                result.url,
                result.structured_data.as_ref().map(|v| v.to_string()),
                result.nlp_data.as_ref().map(|v| v.to_string()),
                serde_json::to_string(&result.extracted.emails).unwrap_or_default(),
                serde_json::to_string(&result.extracted.phones).unwrap_or_default(),
                result.created_at,
            ],
        ).map_err(|e| format!("Failed to save deep result: {}", e))?;
        Ok(())
    }

    pub fn load_deep_results(&self) -> Result<Vec<crate::api::routes::DeepResult>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, job_id, url, structured_data, nlp_data, emails, phones, created_at FROM deep_results ORDER BY created_at DESC"
        ).map_err(|e| format!("Failed to prepare: {}", e))?;

        let results = stmt.query_map([], |row| {
            let sd_str: Option<String> = row.get(3)?;
            let nd_str: Option<String> = row.get(4)?;
            let emails_str: String = row.get(5)?;
            let phones_str: String = row.get(6)?;

            Ok(crate::api::routes::DeepResult {
                id: row.get(0)?,
                job_id: row.get(1)?,
                url: row.get(2)?,
                structured_data: sd_str.and_then(|s| serde_json::from_str(&s).ok()),
                nlp_data: nd_str.and_then(|s| serde_json::from_str(&s).ok()),
                extracted: crate::scraper::extractor::ExtractedData {
                    emails: serde_json::from_str(&emails_str).unwrap_or_default(),
                    phones: serde_json::from_str(&phones_str).unwrap_or_default(),
                },
                created_at: row.get(7)?,
            })
        }).map_err(|e| format!("Failed to query: {}", e))?;

        let mut result = Vec::new();
        for r in results {
            result.push(r.map_err(|e| format!("Failed to read row: {}", e))?);
        }
        Ok(result)
    }

    pub fn delete_job(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM jobs WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| format!("Failed to delete job: {}", e))?;
        Ok(())
    }

    pub fn delete_deep_result(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM deep_results WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| format!("Failed to delete deep result: {}", e))?;
        Ok(())
    }

    pub fn clear_deep_results(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM deep_results", [])
            .map_err(|e| format!("Failed to clear deep results: {}", e))?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch("DELETE FROM jobs; DELETE FROM deep_results;")
            .map_err(|e| format!("Failed to clear database: {}", e))?;
        Ok(())
    }

    pub fn get_deep_result(&self, id: &str) -> Result<crate::api::routes::DeepResult, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, job_id, url, structured_data, nlp_data, emails, phones, created_at FROM deep_results WHERE id = ?1"
        ).map_err(|e| format!("Failed to prepare: {}", e))?;

        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            let sd_str: Option<String> = row.get(3)?;
            let nd_str: Option<String> = row.get(4)?;
            let emails_str: String = row.get(5)?;
            let phones_str: String = row.get(6)?;

            Ok(crate::api::routes::DeepResult {
                id: row.get(0)?,
                job_id: row.get(1)?,
                url: row.get(2)?,
                structured_data: sd_str.and_then(|s| serde_json::from_str(&s).ok()),
                nlp_data: nd_str.and_then(|s| serde_json::from_str(&s).ok()),
                extracted: crate::scraper::extractor::ExtractedData {
                    emails: serde_json::from_str(&emails_str).unwrap_or_default(),
                    phones: serde_json::from_str(&phones_str).unwrap_or_default(),
                },
                created_at: row.get(7)?,
            })
        }).map_err(|e| format!("Failed to query: {}", e))?;

        match rows.next() {
            Some(Ok(r)) => Ok(r),
            _ => Err("Not found".into()),
        }
    }

    pub fn save_schedule(&self, sched: &crate::api::routes::Schedule) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO schedules (id, url, interval_min, config, enabled, last_run, next_run, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                sched.id, sched.url, sched.interval_min as i64,
                serde_json::to_string(&sched.config).unwrap_or_default(),
                sched.enabled as i64,
                sched.last_run, sched.next_run, sched.created_at,
            ],
        ).map_err(|e| format!("Failed to save schedule: {}", e))?;
        Ok(())
    }

    pub fn load_schedules(&self) -> Result<Vec<crate::api::routes::Schedule>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, url, interval_min, config, enabled, last_run, next_run, created_at FROM schedules ORDER BY created_at DESC"
        ).map_err(|e| format!("Failed to prepare: {}", e))?;

        let results = stmt.query_map([], |row| {
            let config_str: String = row.get(3)?;
            Ok(crate::api::routes::Schedule {
                id: row.get(0)?,
                url: row.get(1)?,
                interval_min: row.get::<_, i64>(2)? as u64,
                config: serde_json::from_str(&config_str).unwrap_or_default(),
                enabled: row.get::<_, i64>(4)? != 0,
                last_run: row.get(5)?,
                next_run: row.get(6)?,
                created_at: row.get(7)?,
            })
        }).map_err(|e| format!("Failed to query: {}", e))?;

        let mut result = Vec::new();
        for r in results {
            result.push(r.map_err(|e| format!("Failed to read row: {}", e))?);
        }
        Ok(result)
    }

    pub fn delete_schedule(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM schedules WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| format!("Failed to delete schedule: {}", e))?;
        Ok(())
    }
}
