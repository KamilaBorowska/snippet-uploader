CREATE TABLE logins (
    login_id serial PRIMARY KEY,
    ip inet NOT NULL,
    user_id int NOT NULL REFERENCES users,
    time timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX ON logins (user_id, time DESC);
