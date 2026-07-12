-- Add up migration script here
-- Create Subscriptions Table
CREATE TABLE users(
    id uuid NOT NULL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password TEXT NOT NULL
);