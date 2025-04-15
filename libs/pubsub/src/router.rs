use tokio::sync::broadcast;
use tokio::task::JoinSet;

use crate::message::Message;
use crate::traits::{Handler, Publisher, Subscriber};
use crate::Error;

#[cfg(feature = "bus")]
use crate::bus::Bus;

struct MessageHandler {
    message_channel: broadcast::Receiver<Message>,
    handler: Box<dyn Handler>,
}

impl MessageHandler {
    async fn run(mut self) -> Result<(), String> {
        loop {
            match self.message_channel.recv().await {
                Ok(msg) => {
                    if let Err(e) = self.handler.handle(msg).await {
                        tracing::error!("{:?}", e);
                    }
                }
                Err(_) => todo!(),
            }
        }
    }
}

pub struct Router<P: Publisher, S: Subscriber> {
    publisher: P,
    subsrciber: S,
    handlers: Vec<MessageHandler>,
}

impl<P, S> Router<P, S>
where
    P: Publisher,
    S: Subscriber,
{
    pub fn new(publisher: P, subsrciber: S) -> Self {
        Self {
            publisher,
            subsrciber,
            handlers: vec![],
        }
    }

    pub fn add_handler<H>(mut self, topic: &'static str, handler: H) -> Self
    where
        H: Handler + 'static,
    {
        let message_handler = MessageHandler {
            message_channel: self.subsrciber.subscribe(topic),
            handler: Box::new(handler),
        };

        self.handlers.push(message_handler);

        self
    }

    pub async fn run(mut self) -> Result<(), Error> {
        // 提前获取处理器数量，避免在循环中重复计算
        let handler_count = self.handlers.len();

        if handler_count == 0 {
            return Ok(());
        }
        let mut set = JoinSet::new();

        // 将所有处理器添加到JoinSet中
        for handler in self.handlers.drain(..) {
            set.spawn(async move { handler.run().await });
        }

        // 处理结果
        while let Some(result) = set.join_next().await {
            match result {
                Ok(Ok(())) => {} // 处理器成功完成
                Ok(Err(e)) => {
                    // 清理剩余任务
                    set.abort_all();
                    tracing::error!("{:?}", e);
                }
                Err(e) => {
                    // 清理剩余任务
                    set.abort_all();
                    return Err(Error::HandlerPainc(e.to_string())); // 任务panic
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "bus")]
    pub fn bus(&self) -> Bus<P> {
        Bus::new(self.publisher.clone())
    }
}

#[cfg(feature = "default-pubsub")]
#[cfg(feature = "bus")]
#[cfg(test)]
mod tests {

    use tokio::time::{sleep, Duration};

    use crate::error::HandleError;
    use crate::message::Message;
    use crate::pubsub::PubSub;
    use crate::router::Router;
    use crate::traits::Publisher;

    struct Handler1;

    #[async_trait::async_trait]
    impl crate::traits::Handler for Handler1 {
        async fn handle(&self, msg: Message) -> Result<(), HandleError> {
            println!(
                "handler1: {} -> {}",
                msg.id(),
                String::from_utf8_lossy(&msg.payload())
            );
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_router_publish_and_handling() -> Result<(), Box<dyn std::error::Error>> {
        let pubsub = PubSub::new(100);

        // 创建并配置Router
        let router = Router::new(pubsub.clone(), pubsub.clone())
            .add_handler("topic1", Handler1)
            .add_handler("topic2", |msg: Message| async move {
                println!(
                    "handler2 {} -> {}",
                    msg.id(),
                    String::from_utf8_lossy(&msg.payload())
                );
                Ok(())
            });

        let bus = router.bus();

        // 在后台运行router
        let router_handle = tokio::spawn(async move { router.run().await });

        // 等待router启动
        sleep(Duration::from_millis(100)).await;

        let msg1 = Message::from("hello topic1");

        bus.publish("topic1", msg1).await?;

        // 发布消息到topic2
        let msg2 = Message::from("hello topic2");
        pubsub.publish("topic2", msg2).await?;

        // 终止router
        router_handle.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_router_with_no_handlers() -> Result<(), Box<dyn std::error::Error>> {
        let pubsub = PubSub::new(100);
        let router = Router::new(pubsub.clone(), pubsub.clone());

        // 运行router，应该立即返回Ok
        let result = router.run().await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_router_handler_error() -> Result<(), Box<dyn std::error::Error>> {
        let pubsub = PubSub::new(100);

        let router = Router::new(pubsub.clone(), pubsub.clone())
            .add_handler("topic1", |_| async move {
                Err(HandleError::Wrap("test error".to_string()))
            });

        let bus = router.bus();

        // 在后台运行router
        let router_handle = tokio::spawn(async move { router.run().await });

        // 等待router启动
        sleep(Duration::from_millis(100)).await;

        // 发布消息到topic1
        let msg = Message::from("Test error");

        bus.publish("topic1", msg).await?;

        // 等待错误被处理（但不会导致router终止）
        sleep(Duration::from_millis(200)).await;

        // 终止router
        router_handle.abort();

        Ok(())
    }
}
