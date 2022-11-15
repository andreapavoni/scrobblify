SELECT
  t.id,
  t.name,
  COUNT(*) AS score,
  SUM(s.duration_secs) AS listened_secs
FROM scrobbles AS s
  JOIN artists_tracks AS tt ON s.track_id = tt.track_id
  JOIN artists AS t ON t.id = tt.artist_id
GROUP BY t.id
ORDER BY score DESC, listened_secs DESC
LIMIT 10;