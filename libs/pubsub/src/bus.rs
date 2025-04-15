use crate::{message::Message, traits::Publisher, Error};

#[cfg(feature = "topic")]
use {crate::Topic, serde::Serialize};

#[derive(Clone)]
pub struct Bus<P: Publisher>(P);

impl<P: Publisher> Bus<P> {
    pub fn new(publisher: P) -> Self {
        Self(publisher)
    }

    pub async fn publish(&self, topic: &str, payload: Message) -> Result<(), Error> {
        self.0.publish(topic, payload).await
    }

    #[cfg(feature = "topic")]
    pub async fn send<T: Serialize + Topic>(&self, data: T) -> Result<(), Error> {
        self.publish(T::TOPIC, Message::from(data)).await
    }
}
