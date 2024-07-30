use sqlx::SqlitePool;

pub async fn add_guild(
    pool: &SqlitePool,
    guild_id: i64,
    email_pattern: String,
    role_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO email_verify (guild_id, email_pattern, role_id) VALUES (?, ?, ?)",
        guild_id,
        email_pattern,
        role_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_guild(pool: &SqlitePool, guild_id: i64) -> anyhow::Result<Option<(String, i64)>> {
    let row = sqlx::query!(
        "SELECT email_pattern, role_id FROM email_verify WHERE guild_id = ?",
        guild_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| (row.email_pattern, row.role_id)))
}
