pub mod broadcast;
pub mod collector;
pub mod events;
pub mod listener;
pub mod notifier;
pub mod scope;
pub mod subscriber;
pub mod subscription;

pub use broadcast::*;
pub use collector::*;
pub use events::*;
pub use listener::*;
pub use notifier::*;
pub use scope::*;
pub use subscriber::*;
pub use subscription::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notifier_subscribe_and_broadcast() {
        let notifier = Notifier::new();
        let (id, mut rx) = notifier.register_listener();

        notifier.subscribe(id, Scope::SinkBlueScoreChanged).await;

        notifier
            .notify(Notification::SinkBlueScoreChanged {
                sink_blue_score: 12345,
            })
            .await;

        let received = rx.recv().await.expect("received notification");
        match received {
            Notification::SinkBlueScoreChanged { sink_blue_score } => {
                assert_eq!(sink_blue_score, 12345);
            }
            _ => panic!("unexpected notification"),
        }
    }
}
