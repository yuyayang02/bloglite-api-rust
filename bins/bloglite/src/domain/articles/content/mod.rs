mod error;
pub mod validators;

pub use error::Error;
use std::collections::HashMap;
use validators::*;

#[derive(Clone)]
pub struct FrontMatter {
    pub title: Title,
    pub tags: TagGroup<Tag>,
    pub summary: Summary,
}

#[derive(Clone)]
pub struct Content {
    pub frontmatter: FrontMatter,
    pub hash: String,
    pub body: Body,
    pub rendered_summary: String,
    pub rendered_body: String,
}

pub trait ContentHasher {
    /// 通过输入内容生成hash
    ///
    /// !hash结果不能为空，这点应由外部实现保证
    fn hash(&self, frontmatter: &FrontMatter, body: &Body) -> Result<String, Error>;
}

pub trait ContentParser {
    /// 解析文档内容，返回 front matter 和 正文
    ///
    fn parse<T: AsRef<str>>(&self, raw: T) -> Result<(HashMap<String, String>, String), Error>;
}

pub trait ContentRender {
    /// 将输入内容渲染为html
    ///
    fn render<T: AsRef<str>>(
        &self,
        content: T,
    ) -> impl std::future::Future<Output = Result<String, Error>>;
}

pub struct ContentFactory<P, R, H>
where
    P: ContentParser,
    R: ContentRender,
    H: ContentHasher,
{
    parser: P,
    hasher: H,
    render: R,
}

impl<P, R, H> ContentFactory<P, R, H>
where
    P: ContentParser,
    R: ContentRender,
    H: ContentHasher,
{
    pub fn new(parser: P, hasher: H, render: R) -> Self {
        Self {
            parser,
            hasher,
            render,
        }
    }

    pub async fn process<T: AsRef<str>>(&self, raw_content: T) -> Result<Content, Error> {
        // 阶段 1：解析原始内容
        let (metadata, body) = self.parse_raw_content(raw_content)?;

        // 阶段 2：构建元数据对象
        let frontmatter = self.build_frontmatter(metadata)?;

        // 阶段 3：生成内容哈希
        let hash = self.generate_hash(&frontmatter, &body)?;

        // 阶段 4：渲染最终内容
        let (rendered_body, rendered_summary) =
            self.render_content(&body, &frontmatter.summary).await?;

        Ok(Content {
            frontmatter,
            hash,
            body,
            rendered_summary,
            rendered_body,
        })
    }

    // 阶段 1：原始内容解析
    fn parse_raw_content<T: AsRef<str>>(
        &self,
        raw: T,
    ) -> Result<(HashMap<String, String>, Body), Error> {
        let (metadata, body_str) = self.parser.parse(raw)?;
        let body = Body::new(body_str)?;
        Ok((metadata, body))
    }

    // 阶段 2：元数据构建
    fn build_frontmatter(&self, metadata: HashMap<String, String>) -> Result<FrontMatter, Error> {
        let title = self.extract_title(&metadata)?;
        let summary = self.extract_summary(&metadata)?;
        let tags = self.build_tag_group(&metadata)?;

        Ok(FrontMatter {
            title,
            tags,
            summary,
        })
    }

    fn extract_title(&self, metadata: &HashMap<String, String>) -> Result<Title, Error> {
        metadata
            .get("title")
            .map(Title::new)
            .ok_or(Error::MissingField("title"))?
    }

    fn extract_summary(&self, metadata: &HashMap<String, String>) -> Result<Summary, Error> {
        metadata
            .get("summary")
            .map(Summary::new)
            .ok_or(Error::MissingField("summary"))?
    }

    fn build_tag_group(&self, metadata: &HashMap<String, String>) -> Result<TagGroup<Tag>, Error> {
        TagGroup::new(metadata.get("tags").unwrap_or(&String::new()))
    }

    // 阶段 3：哈希生成
    fn generate_hash(&self, frontmatter: &FrontMatter, body: &Body) -> Result<String, Error> {
        self.hasher.hash(frontmatter, body)
    }

    // 阶段 4：内容渲染
    async fn render_content(
        &self,
        body: &Body,
        summary: &Summary,
    ) -> Result<(String, String), Error> {
        let rendered_body = self.render.render(body).await?;
        let rendered_summary = self.render.render(summary).await?;
        Ok((rendered_body, rendered_summary))
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub fn create_content(title: &str, summary: &str, body: &str, hash: &str) -> Content {
        Content {
            frontmatter: FrontMatter {
                title: validators::Title::new(title).unwrap(),
                summary: validators::Summary::new(summary).unwrap(),
                tags: validators::TagGroup::new("test,123,rust").unwrap(),
            },
            body: validators::Body::new(body).unwrap(),
            hash: hash.to_string(),
            rendered_body: "".to_string(),
            rendered_summary: "".to_string(),
        }
    }

    // 测试辅助模块
    mod test_utils {
        use super::*;
        use std::collections::HashMap;

        // Mock 解析器
        pub struct MockParser {
            metadata: HashMap<String, String>,
            body: String,
        }

        impl MockParser {
            pub fn new(metadata: HashMap<String, String>, body: &str) -> Self {
                Self {
                    metadata,
                    body: body.to_string(),
                }
            }
        }

        impl ContentParser for MockParser {
            fn parse<T: AsRef<str>>(
                &self,
                _: T,
            ) -> Result<(HashMap<String, String>, String), Error> {
                Ok((self.metadata.clone(), self.body.clone()))
            }
        }

        // Mock 渲染器
        pub struct MockRender;

        impl ContentRender for MockRender {
            async fn render<T: AsRef<str>>(&self, content: T) -> Result<String, Error> {
                Ok(format!("[RENDERED]{}", content.as_ref()))
            }
        }

        // Mock 哈希生成器
        pub struct MockHasher;

        impl ContentHasher for MockHasher {
            fn hash(&self, _: &FrontMatter, _: &Body) -> Result<String, Error> {
                Ok("mock_hash".to_string())
            }
        }

        // 创建测试用的元数据
        pub fn test_metadata() -> HashMap<String, String> {
            let mut metadata = HashMap::new();
            metadata.insert("title".to_string(), "Test Title".to_string());
            metadata.insert("summary".to_string(), "Test Summary".to_string());
            metadata.insert("tags".to_string(), "rust,unit-test".to_string());
            metadata
        }
    }

    // 测试验证器
    mod validators_tests {
        use super::*;
        #[test]
        fn title_validations() {
            // 测试空标题
            assert!(matches!(Title::new(""), Err(Error::EmptyField(s)) if s == "title"));

            // 测试超长标题
            let long_title = "a".repeat(Title::MAX_LENGTH + 1);
            assert!(matches!(Title::new(&long_title), Err(Error::TitleTooLong)));

            // 测试合法标题
            let valid_title = "a".repeat(Title::MAX_LENGTH);
            assert!(Title::new(&valid_title).is_ok());
        }

        #[test]
        fn tag_validations() {
            // 测试无效格式
            assert!(matches!(
                Tag::new("invalid tag!"),
                Err(Error::InvalidTagFormat)
            ));

            // 测试有效格式
            assert!(Tag::new("valid-tag123").is_ok());
        }

        #[test]
        fn tag_group_validations() {
            // 测试过多标签
            let many_tags = (0..=TagGroup::MAX_NUM)
                .map(|i| format!("tag{}", i))
                .collect::<Vec<_>>()
                .join(",");
            assert!(matches!(TagGroup::new(&many_tags), Err(Error::TagTooMany)));

            // 测试合法标签数量
            let valid_tags = "rust,test".to_string();
            assert!(TagGroup::new(&valid_tags).is_ok());
        }
    }

    // 测试 ContentFactory
    mod content_factory_tests {
        use super::*;
        use test_utils::*;

        #[tokio::test]
        async fn process_success() {
            // 准备测试数据
            let metadata = test_metadata();
            let parser = MockParser::new(metadata.clone(), "Test Body");
            let factory = ContentFactory::new(parser, MockHasher, MockRender);

            // 执行测试
            let result = factory.process("dummy content").await;
            assert!(result.is_ok());

            let content = result.unwrap();
            assert_eq!(content.frontmatter.title.as_ref(), metadata["title"]);
            assert_eq!(content.rendered_body, "[RENDERED]Test Body");
            assert_eq!(content.hash, "mock_hash");
        }

        #[tokio::test]
        async fn missing_title_field() {
            let mut metadata = test_metadata();
            metadata.remove("title");
            let parser = MockParser::new(metadata, "Test Body");
            let factory = ContentFactory::new(parser, MockHasher, MockRender);

            let result = factory.process("").await;
            assert!(matches!(result, Err(Error::MissingField(s)) if s == "title"));
        }

        #[tokio::test]
        async fn empty_summary_field() {
            let mut metadata = test_metadata();
            metadata.insert("summary".to_string(), "".to_string());
            let parser = MockParser::new(metadata, "Test Body");
            let factory = ContentFactory::new(parser, MockHasher, MockRender);

            let result = factory.process("").await;
            assert!(matches!(result, Err(Error::EmptyField(s)) if s == "summary"));
        }

        #[tokio::test]
        async fn body_too_long() {
            let metadata = test_metadata();
            let long_body = "a".repeat(Body::MAX_LENGTH + 1);
            let parser = MockParser::new(metadata, &long_body);
            let factory = ContentFactory::new(parser, MockHasher, MockRender);

            let result = factory.process("").await;
            assert!(matches!(result, Err(Error::BodyTooLong)));
        }
    }

    mod additional_tests {
        use super::*;
        use test_utils::*;

        #[test]
        fn tag_group_deduplication() {
            let tags = "rust,rust,test,test";
            let tag_group = TagGroup::new(tags).unwrap();
            assert_eq!(
                tag_group.into_iter().len(),
                2,
                "TagGroup should deduplicate tags"
            );
        }

        struct FailingHasher;
        impl ContentHasher for FailingHasher {
            fn hash(&self, _: &FrontMatter, _: &Body) -> Result<String, Error> {
                Err(Error::HashingError("测试".to_string()))
            }
        }

        #[tokio::test]
        async fn content_factory_fails_on_hashing_error() {
            let metadata = test_metadata();
            let parser = MockParser::new(metadata, "Test Body");
            let factory = ContentFactory::new(parser, FailingHasher, MockRender);

            let result = factory.process("").await;
            assert!(matches!(result, Err(Error::HashingError(s)) if s == "测试"));
        }

        struct FailingRender;
        impl ContentRender for FailingRender {
            async fn render<T: AsRef<str>>(&self, _: T) -> Result<String, Error> {
                Err(Error::RenderError("测试".to_string()))
            }
        }

        #[tokio::test]
        async fn content_factory_fails_on_rendering_error() {
            let metadata = test_metadata();
            let parser = MockParser::new(metadata, "Test Body");
            let factory = ContentFactory::new(parser, MockHasher, FailingRender);

            let result = factory.process("").await;
            assert!(matches!(result, Err(Error::RenderError(s)) if s == "测试"));
        }

        struct FailingParser;
        impl ContentParser for FailingParser {
            fn parse<T: AsRef<str>>(
                &self,
                _: T,
            ) -> Result<(HashMap<String, String>, String), Error> {
                Err(Error::ParseError("测试"))
            }
        }

        #[tokio::test]
        async fn content_factory_fails_on_parsing_error() {
            let factory = ContentFactory::new(FailingParser, MockHasher, MockRender);

            let result = factory.process("dummy content").await;
            assert!(matches!(result, Err(Error::ParseError(s)) if s == "测试"));
        }
    }
}
