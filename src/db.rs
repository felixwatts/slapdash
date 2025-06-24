use crate::model::*;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::Connection;

pub(crate) async fn put(db: &mut sqlx::SqliteConnection, series: &str, point: f32) -> anyhow::Result<()> {
    let mut tx = db
        .begin()
        .await
        .map_err(|_| anyhow::anyhow!("Failed to begin transaction"))?;

    // First, insert the series (or ignore if it already exists)
    sqlx::query!("
            INSERT OR IGNORE INTO series (name) 
            VALUES (?)
        ",
        series
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert series: {}", e))?;

    // Then insert the point, using current unix timestamp for time
    sqlx::query!("
        INSERT INTO point (series_id, time, value)
        VALUES (
            (SELECT id FROM series WHERE name = ?),
            strftime('%s','now'),
            ?
        )
    ",
    series,
    point
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| anyhow::anyhow!("Failed to insert point"))?;

    tx.commit().await.map_err(|_| anyhow::anyhow!("Failed to commit transaction"))?;

    Ok(())
}

pub(crate) async fn get(db: &mut sqlx::SqliteConnection, series: &str) -> anyhow::Result<Vec<Point>>{
    let points = sqlx::query_as!(
            Point,
            "
            SELECT 
                datetime(time, 'unixepoch') as `time!: NaiveDateTime`, 
                value as `value!: f32`
            FROM point
            INNER JOIN series ON point.series_id = series.id
            WHERE
                series.name = $1
                AND time > strftime('%s','now') - 86400
            ORDER BY time ASC
            ",
            series
        )
        .fetch_all(db)
        .await
        .map_err(|_| anyhow::anyhow!("Failed to fetch points"))?;

    Ok(points)
}