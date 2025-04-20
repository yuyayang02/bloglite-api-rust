use chrono::{DateTime, Local};

use super::Error;
use crate::domain::articles::{content, events};

pub trait ReadmodelUpdatePolicyProjection<E> {
    type Error;
    fn project<'a, C: sqlx::PgExecutor<'a>>(
        &self,
        event: &E,
        event_time: DateTime<Local>,
        executor: C,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>>;
}

pub struct ReadmodelUpdatePolicy<C: content::ContentRender> {
    render: C,
    db: lib_db::Db,
}

impl<C: content::ContentRender> ReadmodelUpdatePolicy<C> {
    pub fn new(db: lib_db::Db, render: C) -> Self {
        Self { db, render }
    }
}

// 处理文章已创建事件
impl<T> ReadmodelUpdatePolicyProjection<events::ArticleCreated> for ReadmodelUpdatePolicy<T>
where
    T: content::ContentRender + Send + Sync,
{
    type Error = Error;
    async fn project<'a, C: sqlx::PgExecutor<'a>>(
        &self,
        event: &events::ArticleCreated,
        event_time: DateTime<Local>,
        executor: C,
    ) -> Result<(), Self::Error> {
        sqlx::query(
            r#"--sql
            -- 第一部分：插入历史版本
            WITH insert_version AS (
                INSERT INTO article_versions_rm (
                    version, article_id, title, summary, body, tags, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            )
            -- 第二部分：插入/更新主读模型
            INSERT INTO articles_rm (
                title, tags, id, category_id, author, state, current_version,
                rendered_summary, rendered_content, created_at, updated_at, slug, category_name
            )
            VALUES ($3, $6, $2, $8, $9, $10, $1, $11, $12, $7, $7, $13, (SELECT display_name FROM categories WHERE id = $8))
            ON CONFLICT (id) DO UPDATE SET
                category_id = $8,
                author = $9,
                state = $10,
                current_version = $1,
                rendered_summary = $11,
                rendered_content = $12,
                updated_at = $7,
                category_name = (SELECT display_name FROM categories WHERE id = $8)
            "#,
        )
        .bind(&event.current_version) // $1
        .bind(&event.id) // $2
        .bind(&event.title) // $3
        .bind(&event.summary) // $4
        .bind(&event.body) // $5
        .bind(&event.tags) // $6
        .bind(event_time) // $7
        .bind(&event.category_id) // $8
        .bind(&event.author) // $9
        .bind(&event.state) // $10
        .bind(&event.rendered_summary) // $11
        .bind(&event.rendered_body) // $12
        .bind(&event.slug) // $13
        .execute(executor)
        .await?;

        Ok(())
    }
}

// 处理文章已删除事件
impl<T> ReadmodelUpdatePolicyProjection<events::ArticleDeleted> for ReadmodelUpdatePolicy<T>
where
    T: content::ContentRender + Send + Sync,
{
    type Error = Error;
    async fn project<'a, C: sqlx::PgExecutor<'a>>(
        &self,
        event: &events::ArticleDeleted,
        _: DateTime<Local>,
        executor: C,
    ) -> Result<(), Self::Error> {
        sqlx::query(
            r#"--sql
            WITH del_versions AS (
                DELETE FROM article_versions_rm WHERE article_id = $1
            )
            DELETE FROM articles_rm WHERE id = $1
            "#,
        )
        .bind(&event.id)
        .execute(executor)
        .await?;

        Ok(())
    }
}

// 处理文章内容已更新事件
impl<T> ReadmodelUpdatePolicyProjection<events::ArticleContentUpdated> for ReadmodelUpdatePolicy<T>
where
    T: content::ContentRender + Send + Sync,
{
    type Error = Error;
    async fn project<'a, C: sqlx::PgExecutor<'a>>(
        &self,
        event: &events::ArticleContentUpdated,
        event_time: DateTime<Local>,
        executor: C,
    ) -> Result<(), Self::Error> {
        sqlx::query(
            r#"--sql
            WITH insert_version AS (
                INSERT INTO article_versions_rm (
                    version, prev_version, article_id, title, summary, body, tags, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $10)
            )
            UPDATE articles_rm
            SET 
                title = $4,
                tags = $7,
                current_version = $1,
                rendered_summary = $8,
                rendered_content = $9,
                updated_at = $10
            WHERE id = $3
            "#,
        )
        .bind(&event.current_version)
        .bind(&event.parent_version)
        .bind(&event.id)
        .bind(&event.title)
        .bind(&event.summary)
        .bind(&event.body)
        .bind(&event.tags) // 数组类型会自动处理
        .bind(&event.rendered_summary)
        .bind(&event.rendered_body)
        .bind(event_time)
        .execute(executor)
        .await?;

        Ok(())
    }
}

// 处理文章内容已恢复事件
impl<T> ReadmodelUpdatePolicyProjection<events::ArticleContentReverted> for ReadmodelUpdatePolicy<T>
where
    T: content::ContentRender + Send + Sync,
{
    type Error = Error;
    async fn project<'a, C: sqlx::PgExecutor<'a>>(
        &self,
        event: &events::ArticleContentReverted,
        event_time: DateTime<Local>,
        executor: C,
    ) -> Result<(), Self::Error> {
        let (title, summary, body, tags): (String, String, String, Vec<String>) = sqlx::query_as(
            r#"--sql
            SELECT title, summary, body, tags
            FROM article_versions_rm
            WHERE article_id = $1 AND version = $2
            "#,
        )
        .bind(&event.id)
        .bind(&event.current_version)
        .fetch_one(&self.db)
        .await?;

        let rendered_summary = self
            .render
            .render(&summary)
            .await
            .map_err(|e| Error::Exception(e.to_string()))?;

        let rendered_body = self
            .render
            .render(&body)
            .await
            .map_err(|e| Error::Exception(e.to_string()))?;

        sqlx::query(
            r#"--sql
                UPDATE articles_rm
                SET 
                    title = $1,
                    current_version = $2,
                    rendered_summary = $3,
                    rendered_content = $4,
                    tags = $5,
                    updated_at = $6
                WHERE id = $7
                "#,
        )
        .bind(&title)
        .bind(&event.current_version)
        .bind(&rendered_summary)
        .bind(&rendered_body)
        .bind(&tags)
        .bind(event_time)
        .bind(&event.id)
        .execute(executor)
        .await?;

        Ok(())
    }
}

// 处理文章状态更新事件
impl<T> ReadmodelUpdatePolicyProjection<events::ArticleStateChanged> for ReadmodelUpdatePolicy<T>
where
    T: content::ContentRender + Send + Sync,
{
    type Error = Error;
    async fn project<'a, C: sqlx::PgExecutor<'a>>(
        &self,
        event: &events::ArticleStateChanged,
        event_time: DateTime<Local>,
        executor: C,
    ) -> Result<(), Self::Error> {
        sqlx::query(
            r#"--sql
            UPDATE articles_rm
            SET state = $1,
                updated_at = $3
            WHERE id = $2
            "#,
        )
        .bind(&event.state)
        .bind(&event.id)
        .bind(event_time)
        .execute(executor)
        .await?;

        Ok(())
    }
}

// 处理文章分类更新事件
impl<T> ReadmodelUpdatePolicyProjection<events::ArticleCategoryChanged> for ReadmodelUpdatePolicy<T>
where
    T: content::ContentRender + Send + Sync,
{
    type Error = Error;
    async fn project<'a, C: sqlx::PgExecutor<'a>>(
        &self,
        event: &events::ArticleCategoryChanged,
        event_time: DateTime<Local>,
        executor: C,
    ) -> Result<(), Self::Error> {
        sqlx::query(
            r#"--sql
            UPDATE articles_rm
            SET category_id = $1,
                updated_at = $3,
                category_name = (SELECT display_name FROM categories WHERE id = $1)
            WHERE id = $2
            "#,
        )
        .bind(&event.new_category_id)
        .bind(&event.id)
        .bind(event_time)
        .execute(executor)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{_dev_utils, infra};
    use chrono::Duration;
    use events::*;

    #[tokio::test]
    #[ignore = "需要数据库环境，默认不测试"]
    async fn test_readmodel_update_policy() {
        let db = _dev_utils::init_db().await;

        let policy =
            ReadmodelUpdatePolicy::new(db.clone(), infra::domain::ArticleContentRender::default());

        let id = ulid::Ulid::new().to_string();

        let mut tx = db.begin().await.unwrap();

        let base_time = Local::now();

        let v1 = "1na9df".to_string();
        // 1. 文章创建 (私有状态)
        let created = ArticleCreated {
            id: id.clone(),
            slug: "test-article-1".to_string(),
            current_version: v1.clone(), // 示例: "aB3dEf"
            category_id: "private".to_string(),
            author: "user123".to_string(),
            state: 0, // 私有状态
            title: "内部技术文档".to_string(),
            tags: vec!["internal".to_string()],
            body: "## 机密内容\n仅供内部使用".to_string(),
            rendered_body: "<h2>机密内容</h2><p>仅供内部使用</p>".to_string(),
            summary: "内部文档".to_string(),
            rendered_summary: "<p>内部文档</p>".to_string(),
        };
        let t1 = base_time;

        // 2. 状态变更为发布 (5分钟后)
        let state_changed_published = ArticleStateChanged {
            id: id.clone(),
            state: 1, // 发布状态
        };
        let t2 = base_time + Duration::minutes(5);

        // 3. 分类变更 (tech分类，10分钟后)
        let category_changed = ArticleCategoryChanged {
            id: id.clone(),
            old_category_id: "private".to_string(),
            new_category_id: "tech".to_string(),
        };
        let t3 = base_time + Duration::minutes(10);

        let v2 = "2nds9j".to_string();
        // 4. 内容更新 (15分钟后)
        let content_updated = ArticleContentUpdated {
            id: id.clone(),
            parent_version: created.current_version.clone(), // 使用创建时的版本
            current_version: v2,                             // 新版本号
            title: "公开技术文档".to_string(),
            tags: vec!["rust".to_string(), "tutorial".to_string()],
            body: "## 公开内容\n适合所有人阅读".to_string(),
            rendered_body: "<h2>公开内容</h2><p>适合所有人阅读</p>".to_string(),
            summary: "技术教程".to_string(),
            rendered_summary: "<p>技术教程</p>".to_string(),
        };
        let t4 = base_time + Duration::minutes(15);

        // 5. 内容回滚 (20分钟后)
        let content_reverted = ArticleContentReverted {
            id: id.clone(),
            prev_version: content_updated.current_version.clone(), // 回滚前的版本
            current_version: v1,                                   // 新版本号
        };
        let t5 = base_time + Duration::minutes(20);

        // 6. 状态变回私有 (25分钟后)
        let state_changed_private = ArticleStateChanged {
            id: id.clone(),
            state: 0, // 私有状态
        };
        let t6 = base_time + Duration::minutes(25);

        // 7. 文章删除 (30分钟后)
        let deleted = ArticleDeleted { id: id.clone() };
        let t7 = base_time + Duration::minutes(30);

        policy.project(&created, t1, tx.as_mut()).await.unwrap();
        policy
            .project(&state_changed_published, t2, tx.as_mut())
            .await
            .unwrap();
        policy
            .project(&category_changed, t3, tx.as_mut())
            .await
            .unwrap();
        policy
            .project(&content_updated, t4, tx.as_mut())
            .await
            .unwrap();

        tx.commit().await.unwrap();

        let mut tx = db.begin().await.unwrap();

        policy
            .project(&content_reverted, t5, tx.as_mut())
            .await
            .unwrap();
        policy
            .project(&state_changed_private, t6, tx.as_mut())
            .await
            .unwrap();
        policy.project(&deleted, t7, tx.as_mut()).await.unwrap();

        tx.commit().await.unwrap();
    }
}
