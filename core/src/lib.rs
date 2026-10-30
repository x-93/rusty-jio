pub mod core;
pub mod service;

pub use core::{Core, CoreState};
pub use service::{AsyncService, AsyncServiceFuture, CoreError};

#[cfg(test)]
mod tests {
    use super::*;
    use jio_utils::triggers::SingleTrigger;
    use parking_lot::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    struct TestService {
        name: &'static str,
        exit_trigger: SingleTrigger,
        started: Arc<AtomicBool>,
        stopped: Arc<AtomicBool>,
        stop_order: Arc<Mutex<Vec<&'static str>>>,
        should_fail_stop: bool,
    }

    impl TestService {
        fn new(name: &'static str, stop_order: Arc<Mutex<Vec<&'static str>>>) -> Self {
            Self {
                name,
                exit_trigger: SingleTrigger::new(),
                started: Arc::new(AtomicBool::new(false)),
                stopped: Arc::new(AtomicBool::new(false)),
                stop_order,
                should_fail_stop: false,
            }
        }

        fn with_failing_stop(name: &'static str, stop_order: Arc<Mutex<Vec<&'static str>>>) -> Self {
            Self {
                name,
                exit_trigger: SingleTrigger::new(),
                started: Arc::new(AtomicBool::new(false)),
                stopped: Arc::new(AtomicBool::new(false)),
                stop_order,
                should_fail_stop: true,
            }
        }
    }

    impl AsyncService for TestService {
        fn ident(self: Arc<Self>) -> &'static str {
            self.name
        }

        fn start(self: Arc<Self>) -> AsyncServiceFuture {
            Box::pin(async move {
                self.started.store(true, Ordering::SeqCst);
                self.exit_trigger.wait().await;
                Ok(())
            })
        }

        fn signal_exit(self: Arc<Self>) {
            self.exit_trigger.trigger();
        }

        fn stop(self: Arc<Self>) -> AsyncServiceFuture {
            Box::pin(async move {
                self.stopped.store(true, Ordering::SeqCst);
                self.stop_order.lock().push(self.name);
                if self.should_fail_stop {
                    Err(CoreError::ServiceError(self.name, "intentional stop error".into()))
                } else {
                    Ok(())
                }
            })
        }
    }

    #[tokio::test]
    async fn test_core_lifecycle_and_shutdown_order() {
        let core = Core::new();
        let stop_order = Arc::new(Mutex::new(Vec::new()));

        let s1 = Arc::new(TestService::new("service-database", stop_order.clone()));
        let s2 = Arc::new(TestService::new("service-consensus", stop_order.clone()));
        let s3 = Arc::new(TestService::new("service-p2p", stop_order.clone()));

        core.bind(s1.clone());
        core.bind(s2.clone());
        core.bind(s3.clone());

        assert_eq!(core.state(), CoreState::Idle);

        let shutdown_trigger = core.shutdown_trigger();

        // Run daemon in background task
        let core_arc = Arc::new(core);
        let core_clone = core_arc.clone();
        let run_handle = tokio::spawn(async move {
            core_clone.run().await
        });

        // Give tasks a brief moment to start
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(core_arc.state(), CoreState::Running);
        assert!(s1.started.load(Ordering::SeqCst));
        assert!(s2.started.load(Ordering::SeqCst));
        assert!(s3.started.load(Ordering::SeqCst));

        // Signal shutdown
        shutdown_trigger.trigger();

        let result = run_handle.await.unwrap();
        assert!(result.is_ok());

        assert_eq!(core_arc.state(), CoreState::Stopped);
        assert!(s1.stopped.load(Ordering::SeqCst));
        assert!(s2.stopped.load(Ordering::SeqCst));
        assert!(s3.stopped.load(Ordering::SeqCst));

        // Must stop in reverse order of registration: s3 (p2p), s2 (consensus), s1 (database)
        let order = stop_order.lock().clone();
        assert_eq!(order, vec!["service-p2p", "service-consensus", "service-database"]);
    }

    #[tokio::test]
    async fn test_core_cleans_up_even_if_a_service_fails() {
        let core = Core::new();
        let stop_order = Arc::new(Mutex::new(Vec::new()));

        let s1 = Arc::new(TestService::new("s1-base", stop_order.clone()));
        let s2 = Arc::new(TestService::with_failing_stop("s2-failing", stop_order.clone()));
        let s3 = Arc::new(TestService::new("s3-top", stop_order.clone()));

        core.bind(s1.clone());
        core.bind(s2.clone());
        core.bind(s3.clone());

        core.start().unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;

        let res = core.stop().await;
        // Should report the error
        assert!(res.is_err());

        // But ALL services must still have been stopped!
        assert!(s3.stopped.load(Ordering::SeqCst));
        assert!(s2.stopped.load(Ordering::SeqCst));
        assert!(s1.stopped.load(Ordering::SeqCst));
        assert_eq!(core.state(), CoreState::Stopped);
    }
}
