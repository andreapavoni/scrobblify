SELECT
  t.id,
  t.title,
  COUNT(*) AS score,
  SUM(s.duration_secs) AS listened_secs
FROM
  scrobbles AS s
  JOIN tracks AS t ON t.id = s.track_id
GROUP BY
  t.id
ORDER BY
  score DESC;