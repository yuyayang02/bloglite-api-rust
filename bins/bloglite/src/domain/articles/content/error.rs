use super::validators::{Body, Summary, Tag, TagGroup, Title};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("缺少必要字段：'{0}'，请检查文档完整性")]
    MissingField(&'static str),

    #[error("字段'{0}'内容为空，请输入有效内容")]
    EmptyField(&'static str),

    #[error("正文大小超过限制（最大{}MB），请精简内容或拆分文档", {Body::MAX_LENGTH/1024/1024})]
    BodyTooLong,

    #[error("摘要长度超过限制（最大{}KB），请精简要点描述", {Summary::MAX_LENGTH/1024})]
    SummaryTooLong,

    #[error("标题过长（最大{}字节），请保持标题简洁", {Title::MAX_LENGTH})]
    TitleTooLong,

    #[error("单个标签长度超过限制（最大{}字节），请缩短描述", {Tag::MAX_LENGTH})]
    TagTooLong,

    #[error("标签数量超过限制（最多{}个），请删除非必要标签", {TagGroup::MAX_NUM})]
    TagTooMany,

    #[error("标签格式错误，只允许字母、数字和中划线(-)")]
    InvalidTagFormat,

    #[error("文档解析失败，请检查格式是否符合要求")]
    ParseError(&'static str),

    #[error("文件校验失败")]
    HashingError(String),

    #[error("内容生成失败")]
    RenderError(String),
}
