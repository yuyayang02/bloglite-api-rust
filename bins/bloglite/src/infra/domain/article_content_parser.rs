use std::collections::HashMap;

use crate::domain::articles;

pub struct ArticleContentParser;

impl articles::content::ContentParser for ArticleContentParser {
    fn parse<T: AsRef<str>>(
        &self,
        raw: T,
    ) -> Result<(std::collections::HashMap<String, String>, String), articles::content::Error> {
        let input = raw.as_ref().trim();

        // 分类front matter与content
        let (front_matter, content) = parse_front_matter(input)?;

        // 解析front matter为map
        let metadata = convert_yaml_to_map(front_matter)?;

        Ok((metadata, content))
    }
}

/// 提取 front matter 解析逻辑
fn parse_front_matter(input: &str) -> Result<(&str, String), articles::content::Error> {
    const DELIMITER: &str = "---";

    if !input.starts_with(DELIMITER) {
        return Err(articles::content::Error::ParseError("空 front matter 内容"));
    }

    let after_first_delim = &input[DELIMITER.len()..];
    let end_delim_pos = after_first_delim
        .find(DELIMITER)
        .ok_or_else(|| articles::content::Error::ParseError("缺少 front matter 分隔符"))?;

    let (front_matter, remaining) = after_first_delim.split_at(end_delim_pos);
    let content = remaining[DELIMITER.len()..].trim().to_string();

    Ok((front_matter.trim(), content))
}

/// 处理 YAML 转换逻辑
fn convert_yaml_to_map(
    front_matter: &str,
) -> Result<HashMap<String, String>, articles::content::Error> {
    use articles::content::Error;
    use serde_yaml::Value;

    let parsed: Value = serde_yaml::from_str(front_matter)
        .map_err(|_| Error::ParseError("front matter 解析失败"))?;

    let Value::Mapping(map) = parsed else {
        return Err(Error::ParseError("无效的 front matter 内容"));
    };

    map.into_iter()
        .map(|(k, v)| {
            let key = k
                .as_str()
                .ok_or_else(|| Error::ParseError("非法的键类型"))?
                .to_string();

            let value = match v {
                Value::String(s) => s,
                _ => return Err(Error::ParseError("非字符串类型的值")),
            };

            Ok((key, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use articles::content::ContentParser;
    use articles::content::Error;

    #[test]
    fn test_article_content_parser_with_frontmatter() {
        let parser = ArticleContentParser;
        let input = r#"
---
title: Test Title
summary: Test Summary
tags: tag1, tag2
---
Test Body
"#;

        let (metadata, content) = parser.parse(input).unwrap();
        assert_eq!(metadata.get("title"), Some(&"Test Title".to_string()));
        assert_eq!(metadata.get("summary"), Some(&"Test Summary".to_string()));
        assert_eq!(content, "Test Body");
    }

    #[test]
    fn test_article_content_parser_without_frontmatter() {
        let parser = ArticleContentParser;
        let input = "Test Body";

        assert!(
            matches!(parser.parse(input), Err(Error::ParseError(s)) if s == "空 front matter 内容")
        );
    }

    #[test]
    fn test_article_content_parser_invalid_frontmatter() {
        let parser = ArticleContentParser;
        let input = r#"
---
title: Test Title
summary: Test Summary
tags:
  - tag1
  - tag2
"#; // 缺少第二个分隔符

        let result = parser.parse(input);
        assert!(matches!(result, Err(Error::ParseError(_))));
    }

    #[test]
    fn test_article_content_parser_invalid_yaml() {
        let parser = ArticleContentParser;
        let input = r#"
---
title: Test Title
summary: Test Summary
tags:
  - tag1
  - tag2
invalid_yaml
---
Test Body
"#; // 无效的 YAML

        let result = parser.parse(input);
        assert!(matches!(result, Err(Error::ParseError(_))));
    }
}
