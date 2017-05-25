CREATE TABLE files (
    file_id serial PRIMARY KEY,
    user_id int NOT NULL REFERENCES users,
    name varchar NOT NULL,
    UNIQUE (user_id, name)
);
