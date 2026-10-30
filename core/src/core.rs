use crate::service::{AsyncService, CoreError};
use jio_utils::triggers::SingleTrigger;
use parking_lot::Mutex;
use std::future::Future;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreState {
    Idle,
    Running,
    Stopping,
    Stopped,
}

pub type ServiceTaskHandle = (&'static str, JoinHandle<Result<(), CoreError>>);

/// The central daemon runtime that manages background service lifecycles.
pub struct Core {
    services: Mutex<Vec<Arc<dyn AsyncService>>>,
    running_handles: Mutex<Vec<ServiceTaskHandle>>,
    shutdown_trigger: Mutex<SingleTrigger>,
    state: Mutex<CoreState>,
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl Core {
    pub fn new() -> Self {
        Self {
            services: Mutex::new(Vec::new()),
            running_handles: Mutex::new(Vec::new()),
            shutdown_trigger: Mutex::new(SingleTrigger::new()),
            state: Mutex::new(CoreState::Idle),
        }
    }

    /// Access the shutdown trigger to initiate daemon shutdown from any component.
    pub fn shutdown_trigger(&self) -> SingleTrigger {
        self.shutdown_trigger.lock().clone()
    }

    /// Register a new background service with the daemon.
    pub fn bind<S: AsyncService + 'static>(&self, service: Arc<S>) {
        let mut services = self.services.lock();
        services.push(service);
    }

    /// Current lifecycle state.
    pub fn state(&self) -> CoreState {
        *self.state.lock()
    }

    /// Start all registered background services.
    pub fn start(&self) -> Result<(), CoreError> {
        let mut state = self.state.lock();
        if *state == CoreState::Running || *state == CoreState::Stopping {
            return Err(CoreError::AlreadyRunning);
        }

        if self.shutdown_trigger.lock().trigger.is_triggered() {
            *self.shutdown_trigger.lock() = SingleTrigger::new();
        }

        let services = self.services.lock().clone();
        let mut handles = Vec::with_capacity(services.len());

        for service in &services {
            let ident = service.clone().ident();
            let fut = service.clone().start();
            let handle = tokio::spawn(fut);
            handles.push((ident, handle));
        }

        *self.running_handles.lock() = handles;
        *state = CoreState::Running;
        Ok(())
    }

    /// Signal all services to begin exiting.
    pub fn signal_exit(&self) {
        let services = self.services.lock().clone();
        for service in &services {
            service.clone().signal_exit();
        }
        self.shutdown_trigger.lock().trigger.trigger();
    }

    /// Gracefully stop all services and wait for cleanup in reverse order.
    /// Ensures all handles are awaited and all services have `stop()` invoked,
    /// even if one service encounters an error.
    pub async fn stop(&self) -> Result<(), CoreError> {
        {
            let mut state = self.state.lock();
            if *state == CoreState::Stopped {
                return Ok(());
            }
            *state = CoreState::Stopping;
        }

        // 1. Notify all services to exit
        self.signal_exit();

        let mut first_error: Option<CoreError> = None;

        // 2. Await all spawned execution loops
        let handles: Vec<_> = self.running_handles.lock().drain(..).collect();
        for (ident, handle) in handles {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
                Err(e) => {
                    if first_error.is_none() {
                        first_error = Some(CoreError::JoinError(ident, e.to_string()));
                    }
                }
            }
        }

        // 3. Invoke stop() in reverse registration order (LIFO)
        let services: Vec<_> = self.services.lock().iter().cloned().rev().collect();
        for service in services {
            if let Err(e) = service.stop().await {
                first_error.get_or_insert(e);
            }
        }

        *self.state.lock() = CoreState::Stopped;

        if let Some(err) = first_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    /// Start all services, wait for a shutdown signal or trigger, then stop gracefully.
    pub async fn run_until<F>(&self, shutdown_signal: F) -> Result<(), CoreError>
    where
        F: Future<Output = ()> + Send,
    {
        self.start()?;

        let listener = self.shutdown_trigger.lock().listener.clone();
        tokio::select! {
            _ = listener => {},
            _ = shutdown_signal => {
                self.signal_exit();
            }
        }

        self.stop().await
    }

    /// Start all services and run until `shutdown_trigger` is activated.
    pub async fn run(&self) -> Result<(), CoreError> {
        self.run_until(std::future::pending::<()>()).await
    }
}
