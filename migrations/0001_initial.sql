-- Create table for stored data
CREATE TABLE IF NOT EXISTS stored_data (
    id SERIAL PRIMARY KEY, -- Use SERIAL for PostgreSQL
    data TEXT NOT NULL
);
-- Conditional migration for MySQL
-- Uncomment the following lines if using MySQL
-- CREATE TABLE IF NOT EXISTS stored_data (
--     id INTEGER PRIMARY KEY AUTO_INCREMENT, -- Use AUTO_INCREMENT for MySQL
--     data TEXT NOT NULL
-- );

-- Conditional migration for SQLite
-- Uncomment the following lines if using SQLite
-- CREATE TABLE IF NOT EXISTS stored_data (
--     id INTEGER PRIMARY KEY, -- Use INTEGER PRIMARY KEY for SQLite
--     data TEXT NOT NULL
-- );
