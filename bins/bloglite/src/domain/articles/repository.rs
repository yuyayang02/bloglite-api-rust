use pubsub::message::Message;

use super::{Article, ArticleSlug};

pub trait ArticleRepository: Send + Sync {
    type Error;
    /// 查找聚合对象
    fn find(
        &self,
        slug: &ArticleSlug,
    ) -> impl std::future::Future<Output = Result<Option<Article>, Self::Error>>;

    /// 保存聚合与事件
    fn save_all<I>(
        &self,
        article: Article,
        events: I,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        I: IntoIterator<Item = Event> + Send,
        I::IntoIter: Send;
}

pub struct Event(&'static str, pubsub::message::Message);

impl<T: Into<Message> + pubsub::Topic> From<T> for Event {
    fn from(value: T) -> Self {
        Self(T::TOPIC, value.into())
    }
}

impl Event {
    pub fn topic(&self) -> &'static str {
        self.0
    }

    pub fn message(self) -> pubsub::message::Message {
        self.1
    }
}
