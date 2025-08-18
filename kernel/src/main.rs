use autonomy_core::AutonomousAgent;
use common::AppEvent;
use execution_engine::ExecutionEngine;
use libloading::{Library, Symbol};
use metamorphosis_engine::MetamorphosisEngine;
use monitoring_service::{MonitoringConfig, MonitoringService};
use perception_core::run as run_perception_core;
use reasoning_engine::ReasoningEngine;
use resource_monitor::run as run_resource_monitor;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use survival_protocol::SurvivalProtocol;
use tokio::{
    sync::broadcast,
    task::{self, JoinHandle},
    time::{self, Duration},
};

type ModuleRunFn = unsafe extern "C" fn();

struct DynamicModule {
    task_handle: JoinHandle<()>,
}

impl DynamicModule {
    fn new(lib_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let handle = task::spawn_blocking(move || {
            let lib =
                unsafe { Library::new(&lib_path).expect("Library loading failed inside thread") };
            unsafe {
                let run_func: Symbol<ModuleRunFn> = lib
                    .get(b"run_strategy_engine")
                    .expect("Symbol loading failed inside thread");
                run_func();
            }
        });
        Ok(Self {
            task_handle: handle,
        })
    }

    fn shutdown(&self) {
        self.task_handle.abort();
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Kernel starting...");

    let (tx, mut rx) = broadcast::channel::<AppEvent>(100);

    let initial_lib_path = PathBuf::from(if cfg!(target_os = "linux") {
        "target/debug/libstrategy_engine.so"
    } else if cfg!(target_os = "macos") {
        "target/debug/libstrategy_engine.dylib"
    } else {
        "target/debug/strategy_engine.dll"
    });

    let mut strategy_module = Some(DynamicModule::new(initial_lib_path)
        .expect("Failed to load initial strategy engine. Please run 'cargo build -p strategy_engine' first."));
    tracing::info!("Strategy Engine (initial) started.");

    // --- Spawn all other modules correctly ---
    let rm_tx = tx.clone();
    let rm_rx = tx.subscribe();
    task::spawn(run_resource_monitor(rm_tx, rm_rx));
    let pc_tx = tx.clone();
    task::spawn(run_perception_core(pc_tx));
    let mut re = ReasoningEngine::new(tx.clone(), tx.subscribe());
    task::spawn(async move { re.run().await });
    // Note: SshDeployer is private in execution_engine, need to create mock deployer
    struct MockDeployer;
    impl execution_engine::Deployer for MockDeployer {
        fn deploy(&self, _info: common::DeploymentInfo) -> Result<(), Box<dyn std::error::Error>> {
            tracing::info!("[Mock Deployer] Deployment simulated.");
            Ok(())
        }
    }
    let mut ee = ExecutionEngine::new(tx.clone(), tx.subscribe(), Box::new(MockDeployer));
    task::spawn(async move { ee.run().await });
    let mut sp = SurvivalProtocol::new(tx.clone(), tx.subscribe(), 1000.0);
    task::spawn(async move { sp.run().await });
    let mut me = MetamorphosisEngine::new(tx.clone());
    task::spawn(async move { me.run().await });

    // --- Start Autonomous Agent ---
    let binary_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("./kernel"));
    let autonomous_agent = Arc::new(AutonomousAgent::new(binary_path));

    // Initialize the autonomous agent
    if let Err(e) = autonomous_agent.initialize().await {
        tracing::error!("Failed to initialize autonomous agent: {}", e);
    }

    // Start autonomous operations
    let _agent_handle = {
        let agent = autonomous_agent.clone();
        task::spawn(async move {
            if let Err(e) = agent.run().await {
                tracing::error!("Autonomous agent error: {}", e);
            }
        })
    };

    // --- Start Monitoring Service ---
    let monitoring_config = MonitoringConfig {
        port: 8080,
        use_http: true,
    };
    let monitoring_service = Arc::new(MonitoringService::new(monitoring_config));

    // 启动监控服务
    let _monitoring_handle = {
        let service = monitoring_service.clone();
        task::spawn(async move {
            tracing::info!("Starting Rust monitoring HTTP API on port 8080");
            if let Err(e) = service.start().await {
                tracing::error!("Monitoring service error: {}", e);
            }
        })
    };

    // 订阅事件并更新监控数据
    let _monitoring_tx = tx.clone();
    let mut monitoring_rx = tx.subscribe();
    let monitoring_service_clone = monitoring_service.clone();
    task::spawn(async move {
        while let Ok(event) = monitoring_rx.recv().await {
            if let Some(http_service) = monitoring_service_clone.get_http_service() {
                match &event {
                    AppEvent::MarketData(data) => {
                        http_service
                            .update_trading_status(
                                true,
                                Some(data.symbol.clone()),
                                Some(data.price),
                            )
                            .await;
                    }
                    AppEvent::StrategyDecision(
                        common::StrategyDecision::Buy(_, _) | common::StrategyDecision::Sell(_, _),
                    ) => {
                        http_service.record_trade(true).await;
                    }
                    AppEvent::StrategyDecision(_) => {}
                    AppEvent::FinancialUpdate(pnl) => {
                        http_service.update_pnl(*pnl).await;
                    }
                    _ => {}
                }
            }
        }
    });

    tracing::info!("📊 Rust Monitoring API available at: http://localhost:8080");
    tracing::info!("📊 API Endpoints:");
    tracing::info!("   - http://localhost:8080/api/status");
    tracing::info!("   - http://localhost:8080/api/agents");
    tracing::info!("   - http://localhost:8080/api/cluster/status");
    tracing::info!("   - http://localhost:8080/api/metrics");
    tracing::info!("   - http://localhost:8080/api/trading");
    tracing::info!("   - http://localhost:8080/health");

    // --- Kernel Main Loop (Corrected with select!) ---
    let mut file_reader_interval = time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            // Branch 1: Handle internal events
            Ok(event) = rx.recv() => {
                match event {
                    AppEvent::ModuleReadyForHotSwap(lib_path_str) => {
                        tracing::warn!("Hot-swap event received for: {}", lib_path_str);
                        if let Some(old_module) = strategy_module.take() {
                            old_module.shutdown();
                        }

                        match DynamicModule::new(PathBuf::from(lib_path_str)) {
                            Ok(new_module) => {
                                strategy_module = Some(new_module);
                                tracing::info!("New strategy engine started with updated code.");
                            }
                            Err(e) => tracing::error!("Failed to load new dynamic module: {}", e),
                        }
                    }
                    _ => {
                        tracing::debug!(?event, "Kernel observed internal event");
                    }
                }
            }

            // Branch 2: Poll for external events from the dynamic module
            _ = file_reader_interval.tick() => {
                if let Ok(file) = File::open("strategy_output.log") {
                    let reader = BufReader::new(file);
                    for line in reader.lines().map_while(Result::ok) {
                        if let Ok(event) = serde_json::from_str::<AppEvent>(&line) {
                            tracing::info!(?event, "Kernel received event from dynamic module.");
                            if tx.send(event).is_err() {
                                tracing::error!("Failed to broadcast event from dynamic module: No active receivers.");
                            }
                        }
                    }
                    // Clear the file after processing to avoid reprocessing events
                    let _ = std::fs::remove_file("strategy_output.log");
                }
            }
        }
    }
}
