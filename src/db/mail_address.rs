use sqlx::SqlitePool;

pub async fn add_mail_address(
    pool: &SqlitePool,
    guild_id: i64,
    email: String,
) -> anyhow::Result<i64> {
    // get id
    let row = sqlx::query!(
        r#"
        INSERT INTO mail_address (guild_id, email)
        VALUES ($1, $2)
        RETURNING id
        "#,
        guild_id,
        email
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id.unwrap())
}

pub async fn get_all_email(pool: &SqlitePool, guild_id: i64) -> anyhow::Result<Vec<(i64, String)>> {
    let rows = sqlx::query!(
        "SELECT id, email FROM mail_address WHERE guild_id = ?",
        guild_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| (row.id, row.email)).collect())
}

pub async fn delete_mail_address(pool: &SqlitePool, guild_id: i64, id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        "DELETE FROM mail_address WHERE guild_id = ? AND id = ?",
        guild_id,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn exist_mail(pool: &SqlitePool, guild_id: i64, email: String) -> anyhow::Result<bool> {
    let row = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM mail_address
        WHERE guild_id = $1 AND email = $2
        "#,
        guild_id,
        email
    )
    .fetch_one(pool)
    .await?;

    Ok(row.count == 1)
}
