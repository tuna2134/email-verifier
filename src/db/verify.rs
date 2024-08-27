use sqlx::SqlitePool;

pub async fn add_guild(
    pool: &SqlitePool,
    guild_id: i64,
    email_pattern: String,
    role_id: i64,
    channel_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO email_verify (guild_id, email_pattern, role_id, channel_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (guild_id)
        DO UPDATE SET email_pattern = $2, role_id = $3, channel_id = $4
        "#,
        guild_id,
        email_pattern,
        role_id,
        channel_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_guild(
    pool: &SqlitePool,
    guild_id: i64,
) -> anyhow::Result<Option<(String, i64, i64)>> {
    let row = sqlx::query!(
        "SELECT email_pattern, role_id, channel_id FROM email_verify WHERE guild_id = ?",
        guild_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| (row.email_pattern, row.role_id, row.channel_id)))
}
