table! {
    files (file_id) {
        file_id -> Int4,
        user_id -> Int4,
        name -> Varchar,
    }
}

table! {
    logins (login_id) {
        login_id -> Int4,
        ip -> Inet,
        user_id -> Int4,
        time -> Timestamptz,
        successful -> Bool,
    }
}

table! {
    sessions (session_id) {
        session_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    users (user_id) {
        user_id -> Int4,
        name -> Varchar,
        password -> Varchar,
    }
}

joinable!(files -> users (user_id));
joinable!(logins -> users (user_id));
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(files, logins, sessions, users,);
