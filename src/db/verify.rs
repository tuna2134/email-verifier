use sqlx::PgPool;

pub async fn add_guild(
    pool: &PgPool,
    guild_id: i64,
    email_pattern: String,
    role_id: i64,
    channel_id: i64,
    enable_check_mail: bool,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO email_verify (guild_id, email_pattern, role_id, channel_id, enable_check_mail)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (guild_id)
        DO UPDATE SET email_pattern = $2, role_id = $3, channel_id = $4, enable_check_mail = $5
        "#,
        guild_id,
        email_pattern,
        role_id,
        channel_id,
        enable_check_mail
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_guild(
    pool: &PgPool,
    guild_id: i64,
) -> anyhow::Result<Option<(String, i64, i64, bool)>> {
    let row = sqlx::query!(
        "SELECT email_pattern, role_id, channel_id, enable_check_mail FROM email_verify WHERE guild_id = $1",
        guild_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        (
            row.email_pattern,
            row.role_id,
            row.channel_id,
            row.enable_check_mail,
        )
    }))
}
