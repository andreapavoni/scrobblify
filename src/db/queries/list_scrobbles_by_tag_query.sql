WITH
  all_tags AS (
    SELECT
      s.track_id as track_id,
      GROUP_CONCAT(DISTINCT(t.id)) AS tags
    FROM scrobbles AS s
      LEFT JOIN tags_tracks AS tt ON s.track_id = tt.track_id
      LEFT JOIN tags AS t ON tt.tag_id = t.id
    GROUP BY s.track_id
  ),
  all_artists AS (
    SELECT
      s.track_id as track_id,
      GROUP_CONCAT(DISTINCT(a.name)) AS artists
    FROM scrobbles AS s
      LEFT JOIN artists_tracks AS aa ON s.track_id = aa.track_id
      LEFT JOIN artists AS a ON aa.artist_id = a.id
    GROUP BY s.track_id
  )
SELECT
  s.timestamp,
  t.title AS track,
  l.title AS album,
  a.artists AS artists,
  s.duration_secs,
  g.tags AS tags,
  l.cover AS cover
FROM scrobbles AS s
  JOIN tracks AS t ON s.track_id = t.id
  JOIN all_tags AS g ON t.id = g.track_id
  JOIN all_artists AS a ON t.id = a.track_id
  JOIN albums_tracks AS ll ON t.id = ll.track_id
  JOIN albums AS l ON l.id = ll.album_id
  JOIN tags_tracks AS gt ON gt.track_id = t.id
  JOIN tags AS gg ON gg.id = gt.tag_id
WHERE gg.id = ?
ORDER BY s.timestamp DESC;