use crate::model::*;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::Connection;

const MAX_POINTS: i64 = 512;

pub(crate) async fn put_all(db: &mut sqlx::SqliteConnection, points: Vec<(String, NaiveDateTime, f32)>) -> anyhow::Result<()> {
    let mut tx = db
        .begin()
        .await
        .map_err(|_| anyhow::anyhow!("Failed to begin transaction"))?;

    for (series, time, value) in points {
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

        let timestamp = time.and_utc().timestamp();

        // Then insert the point
        sqlx::query!("
                INSERT INTO point (series_id, time, value)
                VALUES (
                    (SELECT id FROM series WHERE name = ?),
                    ?,
                    ?
                )
            ",
            series,
            timestamp,
            value
            )
            .execute(&mut *tx)
            .await
            .map_err(|_| anyhow::anyhow!("Failed to insert point"))?;
    }

    tx.commit().await.map_err(|_| anyhow::anyhow!("Failed to commit transaction"))?;

    Ok(())
}

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

pub(crate) async fn get(db: &mut sqlx::SqliteConnection, series: &str, range_seconds: u32) -> anyhow::Result<Vec<Point>>{
    let points = sqlx::query_as!(
            Point,
            "
            WITH filtered AS (
                SELECT p.time, p.value
                FROM point p
                WHERE p.series_id = (SELECT id FROM series WHERE name = $1 LIMIT 1)
                    AND p.time > strftime('%s','now') - $2
            ),
            bounds AS (
                SELECT MIN(time) AS min_t, MAX(time) AS max_t
                FROM filtered
            ),
            bucketed AS (
                SELECT
                    f.time,
                    f.value,
                    CASE
                        WHEN $3 <= 1 OR b.max_t = b.min_t THEN 0
                        ELSE CAST((f.time - b.min_t) * $3 / (b.max_t - b.min_t + 1) AS INTEGER)
                    END AS bucket
                FROM filtered f
                CROSS JOIN bounds b
            ),
            picked AS (
                SELECT
                    time,
                    value,
                    ROW_NUMBER() OVER (PARTITION BY bucket ORDER BY time ASC) AS rn
                FROM bucketed
            )
            SELECT
                datetime(time, 'unixepoch') as `time!: NaiveDateTime`,
                CAST(value AS REAL) as `value!: f32`
            FROM picked
            WHERE rn = 1
            ORDER BY time ASC
            ",
            series,
            range_seconds,
            MAX_POINTS
        )
        .fetch_all(db)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch points: {}", e.to_string()))?;

    Ok(points)
}