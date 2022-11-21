WITH
  all_tags AS (
    SELECT
      t.id as tag_id,
      s.track_id as track_id,
      COUNT(*) AS tot
    FROM scrobbles AS s
      LEFT JOIN tags_tracks AS tt ON s.track_id = tt.track_id
      LEFT JOIN tags AS t ON tt.tag_id = t.id
		    WHERE s.timestamp >= ?1
          AND s.timestamp <= ?2
    GROUP BY t.id
  ),
  all_scrobbles AS (
    SELECT COUNT(*) as tot
    FROM scrobbles AS s
    WHERE s.timestamp >= ?1
      AND s.timestamp <= ?2
  )
SELECT
  ta.tag_id as tag,
  (ta.tot * 100 / ss.tot) as score
FROM
  scrobbles AS s, all_scrobbles AS ss
  JOIN tags_tracks AS tt ON s.track_id = tt.track_id
  JOIN all_tags AS ta ON ta.tag_id = tt.tag_id
 GROUP BY ta.tag_id
ORDER BY ta.tot DESC
LIMIT ?3