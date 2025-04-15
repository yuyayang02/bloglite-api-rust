use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tokio::sync::broadcast;

use crate::message::Message;
use crate::Error;

#[derive(Clone)]
pub struct PubSub {
    channels: Arc<RwLock<HashMap<&'static str, broadcast::Sender<Message>>>>,
    buffer_size: usize,
}

impl PubSub {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            buffer_size,
        }
    }
}

#[async_trait::async_trait]
impl crate::traits::Publisher for PubSub {
    async fn publish(&self, topic: &str, message: Message) -> Result<(), Error> {
        if let Some(sender) = self.channels.read().unwrap().get(topic) {
            sender.send(message).map_err(|_| Error::NoSubsrcibers)?;
        }
        Ok(())
    }
}

impl crate::traits::Subscriber for PubSub {
    fn subscribe(&mut self, topic: &'static str) -> broadcast::Receiver<Message> {
        let mut channels = self.channels.write().unwrap();

        let sender = channels.entry(topic).or_insert_with(|| {
            let (tx, _) = broadcast::channel(self.buffer_size);
            tx
        });

        sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Publisher, Subscriber};
    use tokio::sync::broadcast::error::RecvError;

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let mut pubsub = PubSub::new(10);

        // Subscribe to a topic
        let mut receiver = pubsub.subscribe("test-topic");

        // Create a test message
        let test_message = Message::from("test payload");

        // Publish the message
        let result = pubsub.publish("test-topic", test_message.clone()).await;
        assert!(result.is_ok(), "Failed to publish message");

        // Receive the message
        let received = receiver.recv().await;
        assert!(received.is_ok(), "Failed to receive message");

        let received_message = received.unwrap();
        assert_eq!(
            received_message.payload(),
            test_message.payload(),
            "Received message doesn't match sent message"
        );
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let mut pubsub = PubSub::new(10);

        // Create multiple subscribers for the same topic
        let mut receiver1 = pubsub.subscribe("shared-topic");
        let mut receiver2 = pubsub.subscribe("shared-topic");

        // Create a test message
        let test_message = Message::from("shared data");

        // Publish the message
        let result = pubsub.publish("shared-topic", test_message.clone()).await;
        assert!(result.is_ok(), "Failed to publish message");

        // Both receivers should get the message
        let received1 = receiver1.recv().await;
        let received2 = receiver2.recv().await;

        assert!(
            received1.is_ok(),
            "First receiver failed to receive message"
        );
        assert!(
            received2.is_ok(),
            "Second receiver failed to receive message"
        );

        assert_eq!(
            received1.unwrap().payload(),
            test_message.payload(),
            "First receiver got incorrect message"
        );
        assert_eq!(
            received2.unwrap().payload(),
            test_message.payload(),
            "Second receiver got incorrect message"
        );
    }

    #[tokio::test]
    async fn test_multiple_topics() {
        let mut pubsub = PubSub::new(10);

        // Subscribe to different topics
        let mut topic1_receiver = pubsub.subscribe("topic1");
        let mut topic2_receiver = pubsub.subscribe("topic2");

        // Create messages for each topic
        let topic1_message = Message::from("topic1 data");
        let topic2_message = Message::from("topic2 data");

        // Publish to both topics
        let result1 = pubsub.publish("topic1", topic1_message.clone()).await;
        let result2 = pubsub.publish("topic2", topic2_message.clone()).await;

        assert!(result1.is_ok(), "Failed to publish to topic1");
        assert!(result2.is_ok(), "Failed to publish to topic2");

        // Each receiver should only get its own topic's message
        let received1 = topic1_receiver.recv().await;
        let received2 = topic2_receiver.recv().await;

        assert!(
            received1.is_ok(),
            "Topic1 receiver failed to receive message"
        );
        assert!(
            received2.is_ok(),
            "Topic2 receiver failed to receive message"
        );

        assert_eq!(
            received1.unwrap().payload(),
            topic1_message.payload(),
            "Topic1 receiver got incorrect message"
        );
        assert_eq!(
            received2.unwrap().payload(),
            topic2_message.payload(),
            "Topic2 receiver got incorrect message"
        );
    }

    #[tokio::test]
    async fn test_buffer_overflow() {
        let mut pubsub = PubSub::new(2);

        let mut receiver = pubsub.subscribe("buffer-topic");

        for i in 0..5 {
            let message = Message::from(format!("message {}", i));
            let _ = pubsub.publish("buffer-topic", message).await;
        }

        assert!(matches!(receiver.recv().await, Err(RecvError::Lagged(n)) if n == 3));

        let message2 = receiver.recv().await.unwrap();
        println!("{}", String::from_utf8_lossy(&message2.payload()));
    }

    #[tokio::test]
    async fn test_nonexistent_topic() {
        let pubsub = PubSub::new(10);

        // Publish to a topic that has no subscribers
        let message = Message::from("test data");
        let result = pubsub.publish("nonexistent", message).await;

        // This should still succeed according to your implementation
        assert!(
            result.is_ok(),
            "Publishing to nonexistent topic should succeed"
        );
    }

    #[tokio::test]
    async fn test_no_subscribers_error() {
        let mut pubsub = PubSub::new(10);

        // Create a topic but then have all subscribers drop
        let topic = "abandoned-topic";
        let receiver = pubsub.subscribe(topic);

        // Explicitly drop the receiver
        drop(receiver);

        // Now try to publish
        let message = Message::from("test data");
        let result = pubsub.publish(topic, message).await;

        // This should result in a NoSubscribers error
        assert!(
            result.is_err(),
            "Expected error when all subscribers have dropped"
        );
        match result {
            Err(Error::NoSubsrcibers) => {} // Expected error
            _ => panic!("Expected NoSubscribers error"),
        }
    }
}
