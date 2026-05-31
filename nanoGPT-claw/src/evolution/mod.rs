//! Evolution Module - Auto Self-Evolution Iteration Core
//!
//! Implements the core self-evolution mechanism that enables NanoGPT-Claw
//! to learn from experiences, benchmark against other agents, and improve
//! its own capabilities over time.
//!
//! 包含 APEX·阿卡西融合完整版 - 全新叠加进化总公式

pub mod bench;
pub mod apex_akashic;
pub mod self_improve;
pub mod super_upgrade;
pub mod apex_self_check;

#[cfg(test)]
mod apex_akashic_tests;

use self::bench::BenchmarkAnalyzer;
use self::apex_akashic::{ApexAkashicCalculator, ApexAkashicResult, format_apex_result};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use rusqlite::{Connection, params};
use tracing::{info, error};
use serde::{Serialize, Deserialize};

/// Evolution engine configuration
pub struct EvolutionConfig {
    pub benchmark_interval_hours: u32,
    pub max_iterations_per_day: u32,
    pub improvement_threshold: f64,
    pub db_path: String,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            benchmark_interval_hours: 24,
            max_iterations_per_day: 10,
            improvement_threshold: 0.05,
            db_path: "nanoGPT-claw.evolution.db".to_string(),
        }
    }
}

/// Evolution event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEvent {
    pub id: String,
    pub timestamp: i64,
    pub event_type: EvolutionEventType,
    pub description: String,
    pub delta_score: f64,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EvolutionEventType {
    Benchmark,
    Improvement,
    CodeRefactor,
    BugFix,
    CapabilityGain,
}

/// Self-evolution engine with SQLite persistence
pub struct EvolutionEngine {
    config: EvolutionConfig,
    events: Arc<RwLock<Vec<EvolutionEvent>>>,
    current_score: Arc<RwLock<f64>>,
    benchmarks: Arc<RwLock<HashMap<String, BenchmarkResult>>>,
    analyzer: BenchmarkAnalyzer,
    db_conn: Arc<RwLock<Option<Connection>>>,
    apex_calculator: Arc<RwLock<ApexAkashicCalculator>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub framework_name: String,
    pub score: f64,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub timestamp: i64,
}

impl EvolutionEngine {
    /// Create new evolution engine
    pub fn new() -> Self {
        Self {
            config: EvolutionConfig::default(),
            events: Arc::new(RwLock::new(Vec::new())),
            current_score: Arc::new(RwLock::new(0.5)),
            benchmarks: Arc::new(RwLock::new(HashMap::new())),
            analyzer: BenchmarkAnalyzer::new(),
            db_conn: Arc::new(RwLock::new(None)),
            apex_calculator: Arc::new(RwLock::new(ApexAkashicCalculator::new())),
        }
    }

    /// Create new evolution engine with config
    pub fn with_config(config: EvolutionConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
            current_score: Arc::new(RwLock::new(0.5)),
            benchmarks: Arc::new(RwLock::new(HashMap::new())),
            analyzer: BenchmarkAnalyzer::new(),
            db_conn: Arc::new(RwLock::new(None)),
            apex_calculator: Arc::new(RwLock::new(ApexAkashicCalculator::new())),
        }
    }

    /// Initialize evolution engine and database
    pub async fn initialize(&self) {
        info!("Initializing evolution engine...");
        info!("  Benchmark interval: {} hours", self.config.benchmark_interval_hours);
        info!("  Max iterations/day: {}", self.config.max_iterations_per_day);
        info!("  Improvement threshold: {:.2}", self.config.improvement_threshold);
        info!("  DB path: {}", self.config.db_path);

        // Initialize database
        if let Err(e) = self.init_db() {
            error!("Failed to initialize evolution database: {}", e);
        }

        // Load historical events from persistent storage
        self.load_events().await;

        info!("Evolution engine initialized ({} historical events)", self.events.read().len());
    }

    /// Initialize SQLite database
    fn init_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::open(&self.config.db_path)?;

        // Enable WAL mode
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Create evolution events table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS evolution_events (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                description TEXT NOT NULL,
                delta_score REAL NOT NULL,
                details TEXT NOT NULL
            )",
            [],
        )?;

        // Create benchmarks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS benchmarks (
                framework_name TEXT PRIMARY KEY,
                score REAL NOT NULL,
                strengths TEXT NOT NULL,
                weaknesses TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        // Create current state table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_ev_timestamp ON evolution_events(timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_ev_type ON evolution_events(event_type)", [])?;

        *self.db_conn.write() = Some(conn);

        Ok(())
    }

    /// Run full benchmark with Φ_APEX*∞ scoring
    pub async fn run_benchmark(&self) -> BenchmarkResult {
        info!("Starting Φ_APEX*∞ benchmark...");
        
        let benchmark = self.analyzer.run_comparison().await;
        
        let result = BenchmarkResult {
            framework_name: benchmark.framework.clone(),
            score: benchmark.overall_score,
            strengths: benchmark.strengths.clone(),
            weaknesses: benchmark.weaknesses.clone(),
            timestamp: current_timestamp(),
        };

        // Save benchmark result to memory and DB
        self.benchmarks.write().insert(
            result.framework_name.clone(),
            result.clone()
        );
        self.save_benchmark(&result).await;

        // Record benchmark event
        let event = EvolutionEvent {
            id: format!("bench_{}", uuid_simple()),
            timestamp: current_timestamp(),
            event_type: EvolutionEventType::Benchmark,
            description: format!("Φ_APEX*∞ score: {:.3}", result.score),
            delta_score: result.score - *self.current_score.read(),
            details: {
                let mut map = HashMap::new();
                map.insert("score".to_string(), format!("{:.3}", result.score));
                for (crit, s) in &benchmark.criteria_scores {
                    map.insert(format!("criteria_{}", crit), format!("{:.3}", s));
                }
                map
            },
        };
        self.record_event(event).await;

        // Update current score
        *self.current_score.write() = result.score;
        self.save_current_score(result.score).await;

        info!("Benchmark complete. Φ_APEX*∞ score: {:.3}", result.score);
        result
    }

    /// 运行 APEX·阿卡西融合进化评估
    pub async fn run_apex_evolution(&self) -> ApexAkashicResult {
        info!("Starting APEX·阿卡西融合进化评估...");
        
        let stats = self.get_stats();
        
        // 更新计算器的维度因子
        {
            let mut calc = self.apex_calculator.write();
            
            // 根据当前统计更新维度
            calc.set_dimension("evolution", stats.total_events as f64 / 100.0).ok();
            calc.set_dimension("value", stats.total_improvement).ok();
            calc.set_dimension("benchmark", stats.current_score).ok();
            
            // 设置惩罚
            let _events_count = stats.total_events as f64;
            let avg_penalty = 0.02;
            calc.set_penalty("token", avg_penalty).ok();
            calc.set_penalty("error", 0.01).ok();
        }
        
        // 计算结果
        let result = {
            let calc = self.apex_calculator.read();
            calc.calculate()
        };
        
        // 记录进化事件
        let event = EvolutionEvent {
            id: format!("apex_{}", uuid_simple()),
            timestamp: current_timestamp(),
            event_type: EvolutionEventType::Benchmark,
            description: format!("APEX·阿卡西融合分数: {:.3}", result.final_score),
            delta_score: result.final_score - *self.current_score.read(),
            details: {
                let mut map = HashMap::new();
                map.insert("apex_score".to_string(), format!("{:.3}", result.final_score));
                map.insert("omega_a".to_string(), format!("{:.3}", result.omega_a));
                map.insert("penalties".to_string(), format!("{:.3}", result.total_penalty));
                map
            },
        };
        self.record_event(event).await;
        
        // 更新当前分数
        *self.current_score.write() = result.final_score;
        self.save_current_score(result.final_score).await;
        
        // 打印格式化结果
        info!("{}", format_apex_result(&result));
        
        result
    }

    /// 获取 APEX 计算器的可变引用
    pub fn apex_calculator(&self) -> Arc<RwLock<ApexAkashicCalculator>> {
        self.apex_calculator.clone()
    }

    /// Record evolution event and persist to DB
    pub async fn record_event(&self, event: EvolutionEvent) {
        {
            let mut events = self.events.write();
            events.push(event.clone());
            info!("Evolution event recorded: {:?}", event.event_type);
        }

        // Save to database (outside lock scope)
        self.save_event(&event).await;
    }

    /// Save event to database
    async fn save_event(&self, event: &EvolutionEvent) {
        let guard = self.db_conn.read();
        if let Some(conn) = guard.as_ref() {
            let details_json = serde_json::to_string(&event.details).unwrap_or_default();
            let event_type_str = format!("{:?}", event.event_type);
            
            let result = conn.execute(
                "INSERT OR REPLACE INTO evolution_events (id, timestamp, event_type, description, delta_score, details)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    event.id,
                    event.timestamp,
                    event_type_str,
                    event.description,
                    event.delta_score,
                    details_json,
                ],
            );

            if let Err(e) = result {
                error!("Failed to save evolution event: {}", e);
            }
        }
    }

    /// Save benchmark to database
    async fn save_benchmark(&self, benchmark: &BenchmarkResult) {
        let guard = self.db_conn.read();
        if let Some(conn) = guard.as_ref() {
            let strengths_json = serde_json::to_string(&benchmark.strengths).unwrap_or_default();
            let weaknesses_json = serde_json::to_string(&benchmark.weaknesses).unwrap_or_default();
            
            let result = conn.execute(
                "INSERT OR REPLACE INTO benchmarks (framework_name, score, strengths, weaknesses, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    benchmark.framework_name,
                    benchmark.score,
                    strengths_json,
                    weaknesses_json,
                    benchmark.timestamp,
                ],
            );

            if let Err(e) = result {
                error!("Failed to save benchmark: {}", e);
            }
        }
    }

    /// Save current score to database
    async fn save_current_score(&self, score: f64) {
        let guard = self.db_conn.read();
        if let Some(conn) = guard.as_ref() {
            let result = conn.execute(
                "INSERT OR REPLACE INTO state (key, value) VALUES (?1, ?2)",
                params!["current_score", &score.to_string()],
            );

            if let Err(e) = result {
                error!("Failed to save current score: {}", e);
            }
        }
    }

    /// Calculate improvement delta
    pub fn calculate_improvement(&self, old_score: f64, new_score: f64) -> f64 {
        if old_score == 0.0 {
            return 0.0;
        }
        (new_score - old_score) / old_score
    }

    /// Process task completion and trigger evolution if needed
    pub async fn process_completion(&self, task_id: &str, success: bool, score_delta: f64) {
        info!("Processing task completion: {} (success: {}, delta: {:.3})", task_id, success, score_delta);

        if success {
            let (_old_score, _new_score, improvement) = {
                let mut current = self.current_score.write();
                let old_score = *current;
                let new_score = (old_score + score_delta).min(1.0);
                *current = new_score;
                let improvement = self.calculate_improvement(old_score, new_score);
                (old_score, new_score, improvement)
            };

            if improvement >= self.config.improvement_threshold {
                let event = EvolutionEvent {
                    id: format!("ev_{}", uuid_simple()),
                    timestamp: current_timestamp(),
                    event_type: EvolutionEventType::Improvement,
                    description: format!("Task {} improved score by {:.2}%", task_id, improvement * 100.0),
                    delta_score: improvement,
                    details: HashMap::new(),
                };
                self.record_event(event).await;
            }
        }
    }

    /// Get evolution statistics
    pub fn get_stats(&self) -> EvolutionStats {
        let events = self.events.read();
        let total_events = events.len();
        let improvements = events.iter()
            .filter(|e| e.event_type == EvolutionEventType::Improvement)
            .count();
        let total_improvement: f64 = events.iter()
            .filter(|e| e.event_type == EvolutionEventType::Improvement)
            .map(|e| e.delta_score)
            .sum();

        EvolutionStats {
            total_events,
            improvement_count: improvements,
            total_improvement,
            current_score: *self.current_score.read(),
            days_active: self.calculate_days_active(),
        }
    }

    /// Get recent evolution events
    pub fn get_recent_events(&self, limit: usize) -> Vec<EvolutionEvent> {
        let events = self.events.read();
        events.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    // Private helpers

    /// Load events from SQLite database
    async fn load_events(&self) {
        let guard = self.db_conn.read();
        if let Some(conn) = guard.as_ref() {
            // Load current score
            if let Ok(score_str) = conn.query_row(
                "SELECT value FROM state WHERE key = ?1",
                params!["current_score"],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(score) = score_str.parse::<f64>() {
                    *self.current_score.write() = score;
                    info!("Loaded current score from DB: {:.3}", score);
                }
            }

            // Load evolution events
            let mut stmt = match conn.prepare("SELECT id, timestamp, event_type, description, delta_score, details FROM evolution_events ORDER BY timestamp ASC") {
                Ok(stmt) => stmt,
                Err(e) => {
                    error!("Failed to prepare statement for loading events: {}", e);
                    return;
                }
            };

            let event_iter = stmt.query_map([], |row| {
                let event_type_str: String = row.get(2)?;
                let details_str: String = row.get(5)?;
                let details: HashMap<String, String> = serde_json::from_str(&details_str).unwrap_or_default();
                
                let event_type = match event_type_str.as_str() {
                    "Benchmark" => EvolutionEventType::Benchmark,
                    "Improvement" => EvolutionEventType::Improvement,
                    "CodeRefactor" => EvolutionEventType::CodeRefactor,
                    "BugFix" => EvolutionEventType::BugFix,
                    "CapabilityGain" => EvolutionEventType::CapabilityGain,
                    _ => EvolutionEventType::Benchmark,
                };

                Ok(EvolutionEvent {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    event_type,
                    description: row.get(3)?,
                    delta_score: row.get(4)?,
                    details,
                })
            });

            match event_iter {
                Ok(iter) => {
                    let mut events = self.events.write();
                    for event_result in iter {
                        if let Ok(event) = event_result {
                            events.push(event);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to load events: {}", e);
                }
            }

            // Load benchmarks
            let mut stmt = match conn.prepare("SELECT framework_name, score, strengths, weaknesses, timestamp FROM benchmarks") {
                Ok(stmt) => stmt,
                Err(e) => {
                    error!("Failed to prepare statement for loading benchmarks: {}", e);
                    return;
                }
            };

            let bench_iter = stmt.query_map([], |row| {
                let strengths_str: String = row.get(2)?;
                let weaknesses_str: String = row.get(3)?;
                
                let strengths: Vec<String> = serde_json::from_str(&strengths_str).unwrap_or_default();
                let weaknesses: Vec<String> = serde_json::from_str(&weaknesses_str).unwrap_or_default();

                Ok(BenchmarkResult {
                    framework_name: row.get(0)?,
                    score: row.get(1)?,
                    strengths,
                    weaknesses,
                    timestamp: row.get(4)?,
                })
            });

            match bench_iter {
                Ok(iter) => {
                    let mut benchmarks = self.benchmarks.write();
                    for bench_result in iter {
                        if let Ok(bench) = bench_result {
                            benchmarks.insert(bench.framework_name.clone(), bench);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to load benchmarks: {}", e);
                }
            }
        }
    }

    fn calculate_days_active(&self) -> u32 {
        let events = self.events.read();
        if events.is_empty() {
            return 0;
        }
        let first_ts = events.first().map(|e| e.timestamp).unwrap_or(0);
        let last_ts = events.last().map(|e| e.timestamp).unwrap_or(0);
        ((last_ts - first_ts) / 86400) as u32
    }
}

impl Default for EvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EvolutionStats {
    pub total_events: usize,
    pub improvement_count: usize,
    pub total_improvement: f64,
    pub current_score: f64,
    pub days_active: u32,
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{:016x}", ns)
}