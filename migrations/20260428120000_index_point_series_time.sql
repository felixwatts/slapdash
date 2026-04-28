-- Range queries always filter by series then time; composite avoids scanning
-- unrelated series when using the time-only index.
CREATE INDEX IF NOT EXISTS idx_point_series_time ON point(series_id, time);
