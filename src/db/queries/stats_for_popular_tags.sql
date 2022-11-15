SELECT
  t.id AS tag,
  COUNT(*) AS score,
  SUM(s.duration_secs) AS listened_secs
FROM scrobbles AS s
  JOIN tags_tracks AS tt ON s.track_id = tt.track_id
  JOIN tags AS t ON t.id = tt.tag_id
WHERE s.timestamp >= ?
  AND s.timestamp <= ?
GROUP BY t.id
ORDER BY score DESC, listened_secs DESC
LIMIT ?;