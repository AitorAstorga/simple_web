use rocket::tokio::sync::RwLock;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use rocket::tokio::fs;
use serde::{Deserialize, Serialize};

use crate::api::git::pull_repo_internal;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AutoPullConfig {
    pub enabled: bool,
    pub interval_minutes: u32,
}

impl Default for AutoPullConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 30,
        }
    }
}

pub struct GitScheduler {
    scheduler: JobScheduler,
    config: Arc<RwLock<AutoPullConfig>>,
    current_job_id: Arc<RwLock<Option<uuid::Uuid>>>,
}

impl GitScheduler {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::new().await?;
        let config = Arc::new(RwLock::new(Self::load_config().await.unwrap_or_default()));
        let current_job_id = Arc::new(RwLock::new(None));

        let git_scheduler = GitScheduler {
            scheduler,
            config,
            current_job_id,
        };

        // Start scheduler
        git_scheduler.scheduler.start().await?;

        // Load existing config and setup job if enabled
        {
            let config_read = git_scheduler.config.read().await;
            if config_read.enabled {
                drop(config_read);
                git_scheduler.setup_auto_pull_job().await?;
            }
        }

        Ok(git_scheduler)
    }

    async fn load_config() -> Result<AutoPullConfig, Box<dyn std::error::Error>> {
        let config_path = "/tmp/auto_pull_config.json";
        if !std::path::Path::new(config_path).exists() {
            return Ok(AutoPullConfig::default());
        }
        
        let content = fs::read_to_string(config_path).await?;
        let config: AutoPullConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    async fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().await;
        let content = serde_json::to_string_pretty(&*config)?;
        fs::write("/tmp/auto_pull_config.json", content).await?;
        Ok(())
    }

    pub async fn update_config(&self, new_config: AutoPullConfig) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔄 Updating auto-pull configuration: enabled={}, interval={}min", 
              new_config.enabled, new_config.interval_minutes);

        // Remove existing job if any
        self.remove_current_job().await?;

        // Update config
        {
            let mut config = self.config.write().await;
            *config = new_config.clone();
        }

        // Save to disk
        self.save_config().await?;

        // Setup new job if enabled
        if new_config.enabled {
            self.setup_auto_pull_job().await?;
        }

        Ok(())
    }

    async fn remove_current_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut job_id_guard = self.current_job_id.write().await;
        if let Some(job_id) = *job_id_guard {
            self.scheduler.remove(&job_id).await?;
            info!("🗑️ Removed existing auto-pull job");
        }
        *job_id_guard = None;
        Ok(())
    }

    async fn setup_auto_pull_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().await;
        let interval = config.interval_minutes;
        drop(config);

        // Create cron expression for every N minutes
        let cron_expr = format!("0 */{} * * * *", interval);
        
        let job = Job::new_async(cron_expr.as_str(), |_uuid, _l| {
            Box::pin(async move {
                info!("🔄 Running scheduled git pull...");
                match pull_repo_internal().await {
                    Ok(status) => {
                        if status.success {
                            info!("✅ Scheduled git pull successful: {}", status.message);
                        } else {
                            warn!("⚠️ Scheduled git pull failed: {}", status.message);
                        }
                    }
                    Err(e) => {
                        error!("❌ Scheduled git pull error: {}", e);
                    }
                }
            })
        })?;

        let job_id = self.scheduler.add(job).await?;
        
        {
            let mut job_id_guard = self.current_job_id.write().await;
            *job_id_guard = Some(job_id);
        }

        info!("⏰ Auto-pull job scheduled every {} minutes", interval);
        Ok(())
    }

    pub async fn get_config(&self) -> AutoPullConfig {
        self.config.read().await.clone()
    }
}

// Global scheduler instance
static GIT_SCHEDULER: tokio::sync::OnceCell<GitScheduler> = tokio::sync::OnceCell::const_new();

pub async fn init_scheduler() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = GitScheduler::new().await?;
    GIT_SCHEDULER.set(scheduler).map_err(|_| "Failed to initialize scheduler")?;
    info!("🚀 Git scheduler initialized");
    Ok(())
}

pub async fn get_scheduler() -> &'static GitScheduler {
    GIT_SCHEDULER.get().expect("Scheduler not initialized")
}