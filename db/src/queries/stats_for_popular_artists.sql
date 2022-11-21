SELECT
  a.id,
  a.name,
	COUNT(DISTINCT(s.track_id)) AS tracks,
	COUNT(DISTINCT(s.timestamp)) AS score
FROM scrobbles AS s
  JOIN artists_tracks AS tt ON s.track_id = tt.track_id
  JOIN artists AS a ON a.id = tt.artist_id
WHERE s.timestamp >= ?1
  AND s.timestamp <= ?2
GROUP BY a.id
ORDER BY score DESC, tracks DESC
LIMIT ?3