use crate::api::notifications::RpcNotification;
use jio_notify::events::Notification;

pub fn rpc_notification_to_notify(n: RpcNotification) -> Notification {
    n
}

pub fn notify_to_rpc_notification(n: Notification) -> RpcNotification {
    n
}
