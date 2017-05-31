CREATE TABLE sessions (
    session_id serial PRIMARY KEY,
    user_id int REFERENCES users NOT NULL
);
