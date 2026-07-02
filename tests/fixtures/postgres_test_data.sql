-- PostgreSQL test data initialization

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    age INTEGER,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create orders table with foreign key
CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    total DECIMAL(10, 2) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index on email
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Create index on user_id in orders
CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);

-- Insert test data
INSERT INTO users (name, email, age, active) VALUES
    ('Alice Johnson', 'alice@example.com', 30, true),
    ('Bob Smith', 'bob@example.com', 25, true),
    ('Charlie Brown', 'charlie@example.com', 35, false),
    ('Diana Prince', 'diana@example.com', 28, true),
    ('Eve Adams', 'eve@example.com', 32, true);

INSERT INTO orders (user_id, total, status) VALUES
    (1, 99.99, 'completed'),
    (1, 49.99, 'completed'),
    (2, 199.99, 'pending'),
    (3, 75.50, 'cancelled'),
    (4, 129.99, 'completed'),
    (4, 89.99, 'pending'),
    (5, 299.99, 'completed');

-- Create a view for testing
CREATE OR REPLACE VIEW user_order_summary AS
SELECT
    u.id as user_id,
    u.name,
    u.email,
    COUNT(o.id) as order_count,
    COALESCE(SUM(o.total), 0) as total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.name, u.email;
