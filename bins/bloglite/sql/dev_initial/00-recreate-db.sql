-- 清除会话
SELECT pg_terminate_backend(pid) FROM pg_stat_activity
WHERE usename = 'bloglite_dev' OR datname = 'bloglite_dev';

-- 删除表、用户
DROP DATABASE IF EXISTS bloglite_dev;
DROP USER IF EXISTS bloglite_dev;

-- 创建表、用户
CREATE USER bloglite_dev PASSWORD 'bloglite_dev';
CREATE DATABASE bloglite_dev OWNER bloglite_dev encoding = 'UTF-8';