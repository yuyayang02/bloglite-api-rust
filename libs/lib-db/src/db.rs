use std::{env, fs, path::PathBuf, str, time::Duration};

use sqlx::postgres::PgPoolOptions;

pub type Db = sqlx::PgPool;

pub async fn init_db_from_env() -> super::Db {
    let conn_url = env::var("DATABASE_URL").ok().unwrap_or_else(|| {
        eprintln!("环境变量: `DATABASE_URL`: NotPresent");
        std::process::exit(1)
    });
    new_db_poll(&conn_url).await.unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1)
    })
}

pub async fn init_db_from_url(conn_url: &str) -> Result<super::Db, sqlx::Error> {
    new_db_poll(conn_url).await
}

// async fn init_db_from_option(conn_url: &str, option: PgPoolOptions) -> super::Db {
//     option.connect(conn_url).await.unwrap()
// }

async fn new_db_poll(conn_url: &str) -> Result<super::Db, sqlx::Error> {
    PgPoolOptions::new()
        .idle_timeout(std::time::Duration::from_secs(240)) // 连接最大空闲时间 5 分钟（超过后自动关闭）
        .max_lifetime(std::time::Duration::from_secs(1500)) // 连接最大生存时间（定期重建 30 分钟
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(2)) // 获取超时时间
        .test_before_acquire(true) // 确保获取时检查连接
        .min_connections(2) // 重要！设置为 0 避免强制保留可能失效的最小连接（默认为0）
        .connect(conn_url)
        .await
}

pub async fn migrate(db: &super::Db, file: &str) -> Result<(), sqlx::Error> {
    let content = std::fs::read_to_string(file)?;

    let sqls: Vec<&str> = content.split(";").collect();
    for sql in sqls {
        if sql.is_empty() {
            continue;
        }
        sqlx::query(sql).execute(db).await?;
    }
    Ok(())
}
