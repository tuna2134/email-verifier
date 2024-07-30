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
