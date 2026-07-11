// MongoDB test data initialization

db = db.getSiblingDB('testdb');

// Create users collection
db.users.drop();
db.users.insertMany([
    {
        name: "Alice Johnson",
        email: "alice@example.com",
        age: 30,
        active: true,
        created_at: new Date("2025-01-15")
    },
    {
        name: "Bob Smith",
        email: "bob@example.com",
        age: 25,
        active: true,
        created_at: new Date("2025-02-10")
    },
    {
        name: "Charlie Brown",
        email: "charlie@example.com",
        age: 35,
        active: false,
        created_at: new Date("2025-03-05")
    },
    {
        name: "Diana Prince",
        email: "diana@example.com",
        age: 28,
        active: true,
        created_at: new Date("2025-04-20")
    },
    {
        name: "Eve Adams",
        email: "eve@example.com",
        age: 32,
        active: true,
        created_at: new Date("2025-05-12")
    }
]);

// Create orders collection
db.orders.drop();
db.orders.insertMany([
    {
        user_email: "alice@example.com",
        total: 99.99,
        status: "completed",
        order_date: new Date("2025-06-01")
    },
    {
        user_email: "alice@example.com",
        total: 49.99,
        status: "completed",
        order_date: new Date("2025-06-05")
    },
    {
        user_email: "bob@example.com",
        total: 199.99,
        status: "pending",
        order_date: new Date("2025-06-10")
    },
    {
        user_email: "charlie@example.com",
        total: 75.50,
        status: "cancelled",
        order_date: new Date("2025-06-15")
    },
    {
        user_email: "diana@example.com",
        total: 129.99,
        status: "completed",
        order_date: new Date("2025-06-20")
    }
]);

// Create indexes
db.users.createIndex({ email: 1 }, { unique: true });
db.users.createIndex({ active: 1, created_at: -1 });
db.orders.createIndex({ user_email: 1, order_date: -1 });

print("MongoDB test data initialized successfully");
