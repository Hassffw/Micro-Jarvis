-- migrations/2024-06-23-000000_create_users/up.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    telegram_id BIGINT NOT NULL,
    name VARCHAR NOT NULL,
    interests TEXT[] NOT NULL,
    goals TEXT[] NOT NULL
);
