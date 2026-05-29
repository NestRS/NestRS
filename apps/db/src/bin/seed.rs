use anyhow::Result;
use sea_orm::Database;

#[tokio::main]
async fn main() -> Result<()> {
    let url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
    let conn = Database::connect(url).await?;
    let inserted = db::seed::run(&conn).await?;
    println!("seed: {inserted} row(s) inserted");
    Ok(())
}
