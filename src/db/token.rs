use sqlx::SqlitePool;

pub async fn set_token(pool: &SqlitePool, user_id: i64, nonce: String) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO token (user_id, nonce)
        VALUES ($1, $2)
        ON CONFLICT (user_id)
        DO UPDATE SET nonce = $2
        "#,
        user_id,
        nonce
    )
    .execute(pool)
    .await?;

    Ok(())
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
