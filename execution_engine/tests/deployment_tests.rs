use common::{AppEvent, DeploymentInfo};
use execution_engine::{ExecutionEngine, Deployer};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::broadcast;

struct MockDeployer {
    was_called: Arc<Mutex<bool>>,
}

impl Deployer for MockDeployer {
    fn deploy(&self, _info: DeploymentInfo) -> Result<(), Box<dyn std::error::Error>> {
        let mut was_called = self.was_called.lock().unwrap();
        *was_called = true;
        Ok(())
    }
}

#[test]
fn test_deployment_event_is_handled() {
    let (tx, mut rx) = broadcast::channel(16);
    let was_called = Arc::new(Mutex::new(false));
    let mock_deployer = MockDeployer { was_called: was_called.clone() };

    let mut engine = ExecutionEngine::new(tx.clone(), rx, Box::new(mock_deployer));

    let deployment_info = DeploymentInfo {
        ip: "127.0.0.1".to_string(),
        remote_user: "test".to_string(),
        private_key_path: "/tmp/test_key".to_string(),
        remote_path: "/tmp/aurelia".to_string(),
    };

    // We need to run the engine in a separate thread because it runs an infinite loop.
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            engine.run().await;
        });
    });

    tx.send(AppEvent::Deploy(deployment_info)).unwrap();

    // Give the engine a moment to process the event.
    thread::sleep(std::time::Duration::from_millis(100));

    let was_called = was_called.lock().unwrap();
    assert!(*was_called, "The deploy method on the mock deployer was not called.");
}