use sqlx::SqlitePool;

pub async fn set_token(
    pool: &SqlitePool,
    user_id: i64,
    nonce: String,
    access_token: String,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO token (user_id, nonce, access_token)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id)
        DO UPDATE SET nonce = $2, access_token = $3
        "#,
        user_id,
        nonce,
        access_token
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_access_token(pool: &SqlitePool, user_id: i64) -> anyhow::Result<String> {
    let row = sqlx::query!(
        r#"
        SELECT access_token
        FROM token
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(row.access_token)
}

pub async fn exist_token(pool: &SqlitePool, user_id: i64, nonce: String) -> anyhow::Result<bool> {
    let row = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM token
        WHERE user_id = $1 AND nonce = $2
        "#,
        user_id,
        nonce
    )
    .fetch_one(pool)
    .await?;

    Ok(row.count == 1)
}
