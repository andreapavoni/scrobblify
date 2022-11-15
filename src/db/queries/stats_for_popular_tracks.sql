SELECT
  t.id,
  t.title,
  COUNT(*) AS score,
  SUM(s.duration_secs) AS listened_secs
FROM scrobbles AS s
  JOIN tracks AS t ON t.id = s.track_id
WHERE s.timestamp >= ?
  AND s.timestamp <= ?
GROUP BY t.id
ORDER BY score DESC, listened_secs DESC
LIMIT ?;