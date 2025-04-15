use crate::domain::articles;

use sha2::Digest;

pub struct ArticleContentHasher;

impl articles::content::ContentHasher for ArticleContentHasher {
    fn hash(
        &self,
        frontmatter: &articles::content::FrontMatter,
        body: &articles::content::validators::Body,
    ) -> Result<String, articles::content::Error> {
        let mut hasher = sha2::Sha256::new();

        hasher.update(body.as_ref());
        hasher.update(frontmatter.title.as_ref());
        hasher.update(frontmatter.summary.as_ref());

        for tag in frontmatter.tags.into_iter() {
            hasher.update(tag.as_ref());
        }
        let result = hasher.finalize();
        Ok(hex::encode(result).chars().take(6).collect::<String>())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use articles::content::ContentHasher;

    #[test]
    fn test_article_content_hasher() {
        let hasher = ArticleContentHasher;
        let frontmatter = articles::content::FrontMatter {
            title: articles::content::validators::Title::new("Test Title").unwrap(),
            summary: articles::content::validators::Summary::new("Test Summary").unwrap(),
            tags: articles::content::validators::TagGroup::new("test1,test2,tag3").unwrap(),
        };
        let body = articles::content::validators::Body::new("Test Body").unwrap();

        let result = hasher.hash(&frontmatter, &body).unwrap();
        assert_eq!(result, hasher.hash(&frontmatter, &body).unwrap()); // 确保哈希结果长度为6
        assert_eq!(result.len(), 6); // 确保哈希结果长度为6
    }
}
