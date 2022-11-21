WITH all_artists AS (
    SELECT
      s.track_id as track_id,
      -- GROUP_CONCAT(DISTINCT(a.name)) AS artists
      json_group_array(DISTINCT(a.name)) as artists
    FROM scrobbles AS s
    LEFT JOIN artists_tracks AS aa ON s.track_id = aa.track_id
    LEFT JOIN artists AS a ON aa.artist_id = a.id
    GROUP BY s.track_id
  )
SELECT
  t.id,
  t.title,
  a.cover as cover,
  COUNT(*) AS score,
  SUM(s.duration_secs) AS listened_secs,
	aa.artists
FROM scrobbles AS s
  JOIN tracks AS t ON t.id = s.track_id
  JOIN albums_tracks AS tt ON tt.track_id = t.id
  JOIN albums AS a ON tt.album_id = a.id
	JOIN all_artists AS aa ON aa.track_id = t.id
WHERE s.timestamp >= ?1
	AND s.timestamp <= ?2
GROUP BY t.id
ORDER BY score DESC, listened_secs DESC
LIMIT ?3;