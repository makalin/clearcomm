use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn create_user(&self, username: &str, password_hash: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3)",
            Uuid::new_v4(),
            username,
            password_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user(&self, username: &str) -> Result<Option<(Uuid, String, String)>> {
        let user = sqlx::query!(
            "SELECT id, username, password_hash FROM users WHERE username = $1",
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(|u| (u.id, u.username, u.password_hash)))
    }

    pub async fn save_message(&self, from_user: &str, to_user: Option<&str>, content: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO messages (id, from_user, to_user, content, created_at) 
             VALUES ($1, $2, $3, $4, NOW())",
            Uuid::new_v4(),
            from_user,
            to_user,
            content
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_messages(&self, username: &str) -> Result<Vec<(String, Option<String>, String, chrono::DateTime<chrono::Utc>)>> {
        let messages = sqlx::query!(
            "SELECT from_user, to_user, content, created_at 
             FROM messages 
             WHERE from_user = $1 OR to_user = $1 OR to_user IS NULL
             ORDER BY created_at DESC
             LIMIT 100",
            username
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(messages
            .into_iter()
            .map(|m| (m.from_user, m.to_user, m.content, m.created_at))
            .collect())
    }
}