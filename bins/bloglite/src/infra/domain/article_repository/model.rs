use std::{collections::HashMap, sync::Arc};

use crate::domain::articles;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct VersionHistoryJson {
    pub version_poll: Vec<String>,    // 版本池
    pub history: Vec<(usize, usize)>, // (当前索引, 父索引)
    pub current_index: usize,         // 当前版本的索引
}

impl From<&articles::version::VersionHistory> for VersionHistoryJson {
    fn from(history: &articles::version::VersionHistory) -> Self {
        // 直接收集所有唯一hash（HashMap的key保证唯一性）
        let mut hashes: Vec<&str> = history.version_history.keys().map(|k| k.as_ref()).collect();

        // 排序以获得稳定顺序（可选）
        hashes.sort_unstable();

        // 构建字符串池和索引映射
        let version_poll = hashes.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let index_map: HashMap<&str, usize> =
            hashes.iter().enumerate().map(|(i, s)| (*s, i)).collect();

        // 查找当前版本索引
        let current_index = index_map[history.current_version_hash.as_ref()];

        // 转换历史记录
        let history = history
            .version_history
            .iter()
            .map(|(hash, ver)| {
                let current_idx = index_map[hash.as_ref()];
                let parent_idx = ver
                    .parent
                    .as_ref()
                    .map(|p| index_map[p.as_ref()])
                    .unwrap_or(usize::MAX);
                (current_idx, parent_idx)
            })
            .collect();

        Self {
            version_poll,
            history,
            current_index,
        }
    }
}

const ERR_INDEX_INVALID: &'static str = "数据映射错误：VersionHistoryJson：索引失效";
const ERR_PARENT_INDEX_INVALID: &'static str = "数据映射错误：VersionHistoryJson：父级索引失效";
const ERR_MISSING_CURRENT_INDEX: &'static str = "数据映射错误：VersionHistoryJson：当前版本缺失";

impl TryFrom<VersionHistoryJson> for articles::version::VersionHistory {
    type Error = &'static str;

    fn try_from(dto: VersionHistoryJson) -> Result<Self, Self::Error> {
        // 预转换字符串池为Arc
        let arc_pool: Vec<Arc<str>> = dto
            .version_poll
            .into_iter()
            .map(|s| Arc::from(s.into_boxed_str()))
            .collect();

        // 构建版本映射
        let mut version_map = HashMap::new();
        for (current_idx, parent_idx) in dto.history {
            // 验证索引有效性
            let current_hash = arc_pool.get(current_idx).ok_or(ERR_INDEX_INVALID)?;

            let parent = if parent_idx == usize::MAX {
                None
            } else {
                Some(
                    arc_pool
                        .get(parent_idx)
                        .ok_or(ERR_PARENT_INDEX_INVALID)?
                        .clone(),
                )
            };

            version_map.insert(
                current_hash.clone(),
                articles::version::Version {
                    hash: current_hash.clone(),
                    parent,
                },
            );
        }

        // 当前版本
        let current_version = arc_pool
            .get(dto.current_index)
            .ok_or(ERR_MISSING_CURRENT_INDEX)?;

        Ok(articles::version::VersionHistory {
            current_version_hash: current_version.clone(),
            version_history: version_map,
        })
    }
}

#[derive(Debug, FromRow)]
pub struct ArticleRow {
    pub id: String,
    pub slug: String,
    pub category: String,
    pub state: i16,
    pub version_history: Json<VersionHistoryJson>,
}

impl lib_db::Table for ArticleRow {
    const TABLE: &'static str = "articles";
}

impl TryFrom<ArticleRow> for articles::Article {
    type Error = lib_db::Error;
    fn try_from(value: ArticleRow) -> Result<Self, Self::Error> {
        let state = match value.state {
            0 => articles::ArticleState::Private,
            1 => articles::ArticleState::Public,
            -1 => articles::ArticleState::Deleted,
            _ => {
                return Err(lib_db::Error::ModelConversionError(format!(
                    "无效的状态值: {}",
                    value.state
                )))
            }
        };

        Ok(articles::ArticleBuilder::only_from_repository(
            value.id,
            value.slug,
            value.category,
            state,
            value
                .version_history
                .0
                .try_into()
                .map_err(|e: &str| Self::Error::ModelConversionError(e.to_string()))?,
        ))
    }
}

impl From<articles::Article> for ArticleRow {
    fn from(value: articles::Article) -> Self {
        let state: i16 = value.state().to_owned().into();

        Self {
            id: value.id().to_string(),
            slug: value.slug().to_string(),
            category: value.category().to_string(),
            state,
            version_history: Json(value.version_history().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::articles::version::{Version, VersionHistory};

    fn new_version_history() -> VersionHistory {
        let version0 = Version::new("v0").unwrap();
        let version1 = Version::new("v1").unwrap();
        let version2 = Version::new("v2").unwrap();
        let version1_5 = Version::new("v1.5").unwrap();
        let mut history = VersionHistory::new(version0).unwrap();
        history.add_version(version1).unwrap();
        history.add_version(version2).unwrap();
        history.rollback_to_version(&"v1").unwrap();
        history.add_version(version1_5).unwrap();
        history
    }

    #[test]
    fn test_version_history_from_into_version_history_json() {
        let history = new_version_history();

        let vhjson = VersionHistoryJson::from(&history);

        let history_into: VersionHistory = vhjson.try_into().unwrap();

        assert_eq!(
            history_into.current_version_hash,
            history.current_version_hash
        );
    }

    #[test]
    fn test_version_history_from_into_version_history_json_for_error_miss_verison() {
        let history = new_version_history();

        let mut vhjson = VersionHistoryJson::from(&history);
        vhjson.current_index = 100; // 做些破坏
        let result = VersionHistory::try_from(vhjson);
        assert!(result.is_err());

        assert!(matches!(result, Err(ERR_MISSING_CURRENT_INDEX)));
    }

    #[test]
    fn test_version_history_from_into_version_history_json_for_error_invaild_verison() {
        let history = new_version_history();

        let mut vhjson = VersionHistoryJson::from(&history);
        vhjson.version_poll = vec![]; // 做些破坏
        let result = VersionHistory::try_from(vhjson);
        assert!(result.is_err());

        assert!(matches!(result, Err(ERR_INDEX_INVALID)));
    }

    #[test]
    fn test_article_row_from_into_article() {
        let history = new_version_history();

        let article_row = ArticleRow {
            id: ulid::Ulid::new().to_string(),
            slug: "slug".to_string(),
            category: "category".to_string(),
            state: 0,
            version_history: Json((&history).into()),
        };

        let article: articles::Article = article_row.try_into().unwrap();

        let articel_row_2: ArticleRow = article.into();

        assert_eq!(articel_row_2.category, "category");
        assert_eq!(articel_row_2.state, 0);
    }

    #[test]
    fn test_article_row_from_into_article_for_state_error() {
        let article_row = ArticleRow {
            id: ulid::Ulid::new().to_string(),
            slug: "slug".to_string(),
            category: "category".to_string(),
            state: 100,
            version_history: Json((&new_version_history()).into()),
        };

        let result = articles::Article::try_from(article_row);

        assert!(matches!(
            result,
            Err(lib_db::Error::ModelConversionError(_))
        ));
    }
}
