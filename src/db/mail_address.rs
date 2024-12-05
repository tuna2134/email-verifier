use sqlx::PgPool;

pub async fn add_mail_address(pool: &PgPool, guild_id: i64, email: String) -> anyhow::Result<i64> {
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

    Ok(row.id as i64)
}

pub async fn get_all_email(pool: &PgPool, guild_id: i64) -> anyhow::Result<Vec<(i64, String)>> {
    let rows = sqlx::query!(
        "SELECT id, email FROM mail_address WHERE guild_id = $1",
        guild_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| (row.id as i64, row.email))
        .collect())
}

pub async fn delete_mail_address(pool: &PgPool, guild_id: i64, id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        "DELETE FROM mail_address WHERE guild_id = $1 AND id = $2",
        guild_id,
        id as i32
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn exist_mail(pool: &PgPool, guild_id: i64, email: String) -> anyhow::Result<bool> {
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

    Ok(row.count == Some(1))
}
