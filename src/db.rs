
use sqlx::PgConnection;
use crate::model::*;

pub(crate) async fn put(db: &mut PgConnection, series: &str, point: f32) -> tide::Result<()> {
    sqlx::query!("
        WITH series_row AS (
            INSERT INTO series (name)
            VALUES ($1)
            ON CONFLICT (name) DO UPDATE
            SET name = EXCLUDED.name -- This is a no-op, just to handle conflict
            RETURNING id
        )
        INSERT INTO point (series_id, time, value)
        VALUES (
            (SELECT id FROM series_row),
            NOW(), -- Assuming you want to use the current time
            $2
        );
    ",
    series,
    point
    )
    .execute(db)
    .await
    .map_err(|e| tide::Error::from_display(e))?;

    Ok(())
}

pub(crate) async fn get(db: &mut PgConnection, series: &str) -> tide::Result<Vec<Point>>{
    let points = sqlx::query_as!(
        Point,
        "
        SELECT time, value
        FROM point
        INNER JOIN series ON point.series_id = series.id
        WHERE 
            series.name = $1
            AND time > NOW() - INTERVAL '24 hours'
        ORDER BY time ASC
        ",
        series
    )
    .fetch_all(db)
    .await
    .map_err(|e| tide::Error::from_display(e))?;

    Ok(points)
}