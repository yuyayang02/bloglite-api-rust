use crate::domain::categories::{self, Category};

#[derive(Debug, sqlx::FromRow)]
struct CategroyRow {
    id: String,
    display_name: String,
}

pub struct CategoryRepository {
    db: lib_db::Db,
}

impl CategoryRepository {
    pub fn new(db: lib_db::Db) -> Self {
        Self { db }
    }
}

impl categories::CategoryRepository for CategoryRepository {
    type Error = lib_db::Error;
    async fn find(&self, id: &impl AsRef<str>) -> Result<Option<Category>, Self::Error> {
        let data = sqlx::query_as::<_, CategroyRow>("select * from categories where id = $1")
            .bind(id.as_ref())
            .fetch_optional(&self.db)
            .await?
            .map(|c| Category::new(c.id, c.display_name));

        Ok(data)
    }
}

impl CategoryRepository {
    pub async fn get_all(&self) -> Result<Vec<Category>, lib_db::Error> {
        Ok(sqlx::query_as::<_, CategroyRow>("select * from categories")
            .fetch_all(&self.db)
            .await?
            .into_iter()
            .map(|c| Category::new(c.id, c.display_name))
            .collect())
    }
}
