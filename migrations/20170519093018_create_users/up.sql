CREATE TABLE users (
    user_id serial PRIMARY KEY,
    name varchar UNIQUE NOT NULL CHECK (name ~ '^[A-Za-z]+$'),
    password varchar NOT NULL
);
