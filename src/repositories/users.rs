use sqlx::{MySql, Pool};

pub async fn create_user(
    pool: &Pool<MySql>,
    email: &str,
    password_hash: &str,
) -> sqlx::Result<u64> {
    let result = sqlx::query!(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES (?, ?)
        "#,
        email,
        password_hash
    )
    .execute(pool)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn get_by_email(pool: &Pool<MySql>, email: &str) -> sqlx::Result<Option<(u64, String)>> {
    let record = sqlx::query!(
        r#"
        SELECT id, password_hash
        FROM users
        WHERE email = ?
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    if let Some(rec) = record {
        Ok(Some((rec.id, rec.password_hash)))
    } else {
        Ok(None)
    }
}
