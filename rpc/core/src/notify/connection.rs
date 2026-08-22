use crate::Notification;

pub type ChannelConnection = jio_notify::connection::ChannelConnection<Notification>;
pub use jio_notify::connection::ChannelType;
