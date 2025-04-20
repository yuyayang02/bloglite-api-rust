-- 文章聚合
CREATE TABLE IF NOT EXISTS articles (
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    category VARCHAR(255) NOT NULL,
    state SMALLINT NOT NULL,
    version_history JSON NOT NULL
);

-- 分类表
CREATE TABLE IF NOT EXISTS categories (
    id VARCHAR(75) PRIMARY KEY NOT NULL,
    display_name VARCHAR(100) NOT NULL
);

-- 事件发件箱
CREATE TABLE IF NOT EXISTS outbox (
    id SERIAL PRIMARY KEY, -- 数据id
    event_id UUID NOT NULL, -- 事件id
    topic VARCHAR(50) NOT NULL, -- 消息队列主题
    payload JSON NOT NULL, -- 事件内容
    occurred_at TIMESTAMP WITH TIME ZONE NOT NULL, -- 发生时间
    retries SMALLINT NOT NULL DEFAULT 0, -- 重试次数
    error TEXT,
    processed BOOLEAN NOT NULL DEFAULT FALSE, -- 是否处理
    processed_at TIMESTAMP WITH TIME ZONE,  -- 处理结果时间
    last_attempt_at TIMESTAMP WITH TIME ZONE -- 最后一次尝试处理时间
);

-- 文章读模型
CREATE TABLE IF NOT EXISTS articles_rm (
    -- slug             TEXT PRIMARY KEY,        -- 唯一标识
    id VARCHAR(26) PRIMARY KEY NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,

    category_id      TEXT NOT NULL,           -- 当前分类
    category_name      TEXT NOT NULL,           -- 冗余字段

    author           VARCHAR(50) NOT NULL,
    state            SMALLINT NOT NULL,       -- 状态（0草稿,1已发布,-1已删除）
    current_version  TEXT NOT NULL,           -- 当前内容版本

    title TEXT NOT NULL,
    tags TEXT[] NOT NULL,

    rendered_summary TEXT NOT NULL,           -- 渲染后的 summary
    rendered_content TEXT NOT NULL,           -- 渲染后的 content

    created_at       TIMESTAMPTZ NOT NULL,    -- 首次创建时间
    updated_at       TIMESTAMPTZ NOT NULL    -- 最后更新时间
);

CREATE INDEX IF NOT EXISTS idx_articles_rm_slug ON articles_rm(slug);

-- 文章历史版本读模型
CREATE TABLE IF NOT EXISTS article_versions_rm (
    id SERIAL PRIMARY KEY,
    prev_version TEXT DEFAULT NULL,
    version TEXT NOT NULL,

    article_id TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    body TEXT NOT NULL,
    tags TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
