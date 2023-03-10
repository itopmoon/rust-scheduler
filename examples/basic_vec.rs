use chrono::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::time::Duration;
use woddle::{async_trait, Job, JobConfig, JobRunner, RunnerConfig};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
struct MyJobContext {
    pub name: String,
}

#[derive(Clone)]
struct MyJob {
    ctx: MyJobContext,
    config: JobConfig,
}

#[async_trait]
impl Job for MyJob {
    async fn run(&self) {
        log::info!("starting  my job!");
        tokio::time::sleep(Duration::from_secs(2)).await;
        let val = COUNTER.fetch_add(1, SeqCst);
        log::info!("job context: {:?}", self.ctx);
        log::info!("job done: time: {}, counter: {}", Utc::now(), val);
    }

    fn get_config(&self) -> &JobConfig {
        &self.config
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let job_cfg = JobConfig::new("my_job", "someSyncKey").interval(Duration::from_secs(5));
    let my_job = MyJob {
        ctx: MyJobContext {
            name: "my context".to_string(),
        },
        config: job_cfg,
    };

    let job_cfg2 = JobConfig::new("my_job2", "someSyncKey2").interval(Duration::from_secs(5));
    let my_job2 = MyJob {
        ctx: MyJobContext {
            name: "my context 2".to_string(),
        },
        config: job_cfg2,
    };

    let config = RunnerConfig::default().check_interval(Duration::from_millis(100));
    let job_runner = JobRunner::new_with_vec(config, vec![my_job, my_job2]);

    tokio::spawn(async move {
        if let Err(e) = job_runner.start().await {
            log::error!("error: {}", e);
        }
    });

    tokio::time::sleep(Duration::from_secs(11)).await;

    assert!(COUNTER.load(SeqCst) == 4);
    log::info!("Success!");
}
