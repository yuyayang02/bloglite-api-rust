use crate::domain::articles;

use super::error::Error;
use std::collections::HashSet;

macro_rules! impl_strings_traits {
    ($($name:ident),+) => {
        $(
            impl From<$name> for String {
                fn from(field: $name) -> Self {
                    field.0
                }
            }
            impl AsRef<str> for $name {
                fn as_ref(&self) -> &str {
                    &self.0
                }
            }
            impl std::ops::Deref for  $name {
                type Target = str;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        )+
    };
}

macro_rules! field {
    ($name:ident, $max:expr, $max_err:expr $(, $role_fn:expr)?) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct $name(String);

        impl $name {
            pub const MAX_LENGTH: usize = $max;

            pub fn new<T: AsRef<str>>(value: T) -> Result<Self, Error> {
                let value = value.as_ref();
                let len = value.len();

                if len > Self::MAX_LENGTH {
                    return Err($max_err);
                }

                let v = Self(value.to_string());
                v.validate()?;

                Ok(v)
            }


            pub fn validate(&self) -> Result<(), Error> {
                $( $role_fn(self)?; )?
                Ok(())
            }

        }
    };
}

field!(Title, 800, Error::TitleTooLong, |title: &Title| {
    if title.as_ref().is_empty() {
        return Err(articles::content::Error::EmptyField("title"));
    };
    Ok(())
});

field!(
    Summary,
    { lib_utils::consts::kb(1) },
    Error::SummaryTooLong,
    |summary: &Summary| {
        if summary.is_empty() {
            Err(articles::content::Error::EmptyField("summary"))
        } else {
            Ok(())
        }
    }
);

field!(Body, { lib_utils::consts::mb(2) }, Error::BodyTooLong);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Tag(String);

impl Tag {
    pub const MAX_LENGTH: usize = 20;
    /// 创建一个 Tag，并验证其内容：
    /// - 不能为空
    /// - 只允许字母、数字和连字符（-），其中字母和数字支持中文
    pub fn new<T: AsRef<str>>(value: T) -> Result<Self, Error> {
        let value = value.as_ref();
        let len = value.len();

        if len > Self::MAX_LENGTH {
            return Err(Error::TagTooLong);
        }
        // 使用 .all() 判断每个字符是否符合要求
        if !value.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(Error::InvalidTagFormat);
        }

        Ok(Self(value.to_string()))
    }
}
impl_strings_traits!(Title, Summary, Body, Tag);

#[derive(Debug, Clone)]
pub struct TagGroup<T>(HashSet<T>);

impl TagGroup<Tag> {
    pub const MAX_NUM: usize = 4;

    pub fn new<T: AsRef<str>>(tags: T) -> Result<Self, Error> {
        // 将输入字符串转换为 &str，并按逗号分割，再逐个解析成 Tag
        let parsed_tags: HashSet<Tag> = tags
            .as_ref()
            .split(',')
            .map(|tag_str| Tag::new(tag_str.trim()))
            .collect::<Result<HashSet<Tag>, _>>()?;

        // 检查标签数量是否超过最大值
        if parsed_tags.len() > Self::MAX_NUM {
            return Err(Error::TagTooMany);
        }

        Ok(TagGroup(parsed_tags))
    }
}

impl<'a> IntoIterator for &'a TagGroup<Tag> {
    type Item = &'a Tag;
    type IntoIter = std::collections::hash_set::Iter<'a, Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<TagGroup<Tag>> for Vec<String> {
    fn from(value: TagGroup<Tag>) -> Self {
        value
            .into_iter()
            .map(|tag| tag.as_ref().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --------- Title 测试 ---------
    #[test]
    fn test_title_valid() {
        let title = Title::new("A valid title").unwrap();
        assert_eq!(title.as_ref(), "A valid title");
    }

    #[test]
    fn test_title_empty() {
        let err = Title::new("").unwrap_err();
        match err {
            Error::EmptyField(field) => assert_eq!(field, "title"),
            _ => panic!("预期 EmptyField 错误"),
        }
    }

    #[test]
    fn test_title_too_long() {
        // Title 最大长度为 800 字节
        let long_str = "a".repeat(801);
        let err = Title::new(long_str).unwrap_err();
        assert!(matches!(err, Error::TitleTooLong));
    }

    // --------- Summary 测试 ---------
    #[test]
    fn test_summary_valid() {
        let summary = Summary::new("A valid summary").unwrap();
        assert_eq!(summary.as_ref(), "A valid summary");
    }

    #[test]
    fn test_summary_empty() {
        let err = Summary::new("").unwrap_err();
        match err {
            Error::EmptyField(field) => assert_eq!(field, "summary"),
            _ => panic!("预期 EmptyField 错误"),
        }
    }

    #[test]
    fn test_summary_too_long() {
        // 假定 Summary::MAX_LENGTH 为 common::constants::kb(1) = 1024 字节
        let long_str = "a".repeat(1025);
        let err = Summary::new(long_str).unwrap_err();
        assert!(matches!(err, Error::SummaryTooLong));
    }

    // --------- Body 测试 ---------
    #[test]
    fn test_body_valid() {
        let body = Body::new("Some valid body content").unwrap();
        assert_eq!(body.as_ref(), "Some valid body content");
    }

    #[test]
    fn test_body_too_long() {
        // 假定 Body::MAX_LENGTH 为 common::constants::mb(2) = 2 * 1024 * 1024 字节
        let long_str = "a".repeat(Body::MAX_LENGTH + 1);
        let err = Body::new(long_str).unwrap_err();
        assert!(matches!(err, Error::BodyTooLong));
    }

    // --------- Tag 测试 ---------
    #[test]
    fn test_tag_valid() {
        let tag = Tag::new("valid-tag123").unwrap();
        assert_eq!(tag.as_ref(), "valid-tag123");
    }

    #[test]
    fn test_tag_too_long() {
        let long_tag = "a".repeat(21); // 超出最大长度 20
        let err = Tag::new(long_tag).unwrap_err();
        assert!(matches!(err, Error::TagTooLong));
    }

    #[test]
    fn test_tag_invalid_format() {
        // tag 中包含空格，应该被认为格式非法
        let err = Tag::new("invalid tag").unwrap_err();
        assert!(matches!(err, Error::InvalidTagFormat));
    }

    // --------- TagGroup 测试 ---------
    #[test]
    fn test_tag_group_valid() {
        let input = "tag1, tag2,tag3";
        let tag_group = TagGroup::new(input).unwrap();
        let tags: Vec<&Tag> = tag_group.into_iter().collect();
        // 预期有 3 个标签（去掉空白后）
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn test_tag_group_too_many() {
        // 超出最大允许标签数 4
        let input = "tag1,tag2,tag3,tag4,tag5";
        let err = TagGroup::new(input).unwrap_err();
        assert!(matches!(err, Error::TagTooMany));
    }
}
