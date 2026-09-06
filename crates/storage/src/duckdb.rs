use chrono::Utc;
use serde::{Deserialize, Serialize};

/// 调用日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallLog {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub task_type: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub latency_ms: u64,
    pub is_success: bool,
    pub error_type: Option<String>,
    pub cost_usd: f64,
    pub created_at: chrono::DateTime<Utc>,
}

/// 模型画像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub provider: String,
    pub model: String,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub success_rate: f64,
    pub avg_cost_per_token: f64,
    pub quality_score: f64,
    pub total_calls: u64,
    pub updated_at: chrono::DateTime<Utc>,
}

/// 分析数据库 (DuckDB)
pub struct AnalyticsDB {
    conn: std::sync::Mutex<duckdb::Connection>,
}

impl AnalyticsDB {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conn = duckdb::Connection::open(path)?;
        let db = Self {
            conn: std::sync::Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS call_logs (
                id TEXT PRIMARY KEY,
                provider VARCHAR,
                model VARCHAR,
                task_type VARCHAR,
                input_tokens INTEGER,
                output_tokens INTEGER,
                latency_ms INTEGER,
                is_success BOOLEAN,
                error_type VARCHAR,
                cost_usd DOUBLE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS model_profiles (
                provider VARCHAR,
                model VARCHAR,
                p50_latency_ms DOUBLE,
                p95_latency_ms DOUBLE,
                success_rate DOUBLE,
                avg_cost_per_token DOUBLE,
                quality_score DOUBLE,
                total_calls INTEGER,
                updated_at TIMESTAMP,
                PRIMARY KEY (provider, model)
            );

            CREATE TABLE IF NOT EXISTS cost_stats (
                provider VARCHAR,
                model VARCHAR,
                date DATE,
                total_calls INTEGER,
                total_input_tokens INTEGER,
                total_output_tokens INTEGER,
                total_cost DOUBLE,
                PRIMARY KEY (provider, model, date)
            );",
        )?;
        Ok(())
    }

    /// 记录一次调用
    pub fn log_call(&self, log: &CallLog) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO call_logs (id, provider, model, task_type, input_tokens, output_tokens, latency_ms, is_success, error_type, cost_usd, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            duckdb::params![
                log.id, log.provider, log.model, log.task_type.clone().unwrap_or_default(),
                log.input_tokens as i32, log.output_tokens as i32,
                log.latency_ms as i32, log.is_success as bool,
                log.error_type.clone().unwrap_or_default(), log.cost_usd,
                log.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// 查询模型画像
    pub fn query_model_profiles(&self) -> anyhow::Result<Vec<ModelProfile>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT provider, model, p50_latency_ms, p95_latency_ms, success_rate, avg_cost_per_token, quality_score, total_calls, updated_at
             FROM model_profiles"
        )?;
        let rows = stmt.query_map([], |row| {
            let updated: String = row.get(8)?;
            Ok(ModelProfile {
                provider: row.get(0)?,
                model: row.get(1)?,
                p50_latency_ms: row.get(2)?,
                p95_latency_ms: row.get(3)?,
                success_rate: row.get(4)?,
                avg_cost_per_token: row.get(5)?,
                quality_score: row.get(6)?,
                total_calls: row.get(7)?,
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_default(),
            })
        })?;

        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(row?);
        }
        Ok(profiles)
    }

    /// 更新模型画像
    pub fn update_model_profile(
        &self,
        provider: &str,
        model: &str,
        latency_ms: u64,
        success: bool,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO model_profiles (provider, model, p50_latency_ms, p95_latency_ms, success_rate, avg_cost_per_token, quality_score, total_calls, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0.0, 0.0, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(provider, model) DO UPDATE SET
                p50_latency_ms = (p50_latency_ms * total_calls + ?3) / (total_calls + 1),
                total_calls = total_calls + 1,
                success_rate = (success_rate * total_calls + CASE WHEN ?5 THEN 1.0 ELSE 0.0 END) / (total_calls + 1),
                updated_at = CURRENT_TIMESTAMP",
            duckdb::params![provider, model, latency_ms as f64, latency_ms as f64, success],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duckdb_init() {
        let db = AnalyticsDB::new(":memory:").unwrap();
        let profiles = db.query_model_profiles().unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_log_call() {
        let db = AnalyticsDB::new(":memory:").unwrap();
        let log = CallLog {
            id: uuid::Uuid::new_v4().to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            task_type: Some("chat".to_string()),
            input_tokens: 10,
            output_tokens: 20,
            latency_ms: 500,
            is_success: true,
            error_type: None,
            cost_usd: 0.001,
            created_at: Utc::now(),
        };
        db.log_call(&log).unwrap();
    }
}
