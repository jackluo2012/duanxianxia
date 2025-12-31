-- db/init_postgres.sql

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    plan VARCHAR(20) DEFAULT 'free',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- 自选股表
CREATE TABLE IF NOT EXISTS user_watchlist (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    code VARCHAR(6) NOT NULL,
    added_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, code)
);

-- 插入测试用户
-- 密码: password123 (bcrypt hash)
INSERT INTO users (username, email, password_hash, plan) VALUES
('testuser', 'test@example.com', '$2b$12$bMlWvJ0z/L/.wUzLZbWm2.4tJYsW5udpfj4iRJyuHUZc4.6oAPKyy', 'free')
ON CONFLICT (username) DO NOTHING;
