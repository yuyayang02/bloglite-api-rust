/// CQRS 命令处理器
// #[async_trait::async_trait]
pub trait CommandHandler<Output = ()> {
    type Command;
    type Error;
    fn handle(
        &self,
        cmd: Self::Command,
    ) -> impl std::future::Future<Output = Result<Output, Self::Error>> + Send;
}

/// CQRS 查询处理器
pub trait QueryHandler {
    type Query;
    type Result;
    type Error;
    fn handle(
        &self,
        query: Self::Query,
    ) -> impl std::future::Future<Output = Result<Self::Result, Self::Error>> + Send;
}
