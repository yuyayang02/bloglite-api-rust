use tokio::sync::broadcast;

use crate::error::HandleError;
use crate::message::Message;

use crate::Error;

#[async_trait::async_trait]
pub trait Publisher: Send + Sync + Clone {
    async fn publish(&self, topic: &str, msg: Message) -> Result<(), Error>;
}

pub trait Subscriber: Send + Sync {
    fn subscribe(&mut self, topic: &'static str) -> broadcast::Receiver<Message>;
}

#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, msg: Message) -> Result<(), HandleError>;
}

#[async_trait::async_trait]
impl<F, Fut> Handler for F
where
    F: Fn(Message) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), HandleError>> + Send + 'static,
{
    async fn handle(&self, msg: Message) -> Result<(), HandleError> {
        self(msg).await
    }
}
