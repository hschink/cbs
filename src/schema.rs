table! {
    bike_translatables (id) {
        id -> Int4,
        bike_id -> Int4,
        locale -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        url -> Nullable<Varchar>,
    }
}

table! {
    bikes (id) {
        id -> Int4,
    }
}

table! {
    rent_details (id) {
        id -> Int4,
        rent_id -> Nullable<Int4>,
        encrypted_details -> Text,
    }
}

table! {
    rents (id) {
        id -> Int4,
        token_id -> Int4,
        bike_id -> Int4,
        created_at -> Timestamptz,
        start_timestamp -> Timestamptz,
        end_timestamp -> Timestamptz,
        revocation_timestamp -> Nullable<Timestamptz>,
    }
}

table! {
    token_challenge_translatables (id) {
        id -> Int4,
        token_challenge_id -> Int4,
        locale -> Varchar,
        question -> Text,
        answer_hash -> Varchar,
        url -> Nullable<Varchar>,
    }
}

table! {
    token_challenges (id) {
        id -> Int4,
    }
}

table! {
    tokens (id) {
        id -> Int4,
        uuid -> Uuid,
        created_at -> Timestamptz,
    }
}

joinable!(bike_translatables -> bikes (bike_id));
joinable!(rent_details -> rents (rent_id));
joinable!(rents -> bikes (bike_id));
joinable!(rents -> tokens (token_id));
joinable!(token_challenge_translatables -> token_challenges (token_challenge_id));

allow_tables_to_appear_in_same_query!(
    bike_translatables,
    bikes,
    rent_details,
    rents,
    token_challenge_translatables,
    token_challenges,
    tokens,
);
