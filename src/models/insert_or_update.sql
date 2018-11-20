INSERT INTO items (id, path, content, ranking, created, modified)
VALUES (
  $1, text2ltree($2), $3, $4, TIMESTAMP 'now', TIMESTAMP 'now'
)
ON CONFLICT (id) DO UPDATE
  SET
    content = EXCLUDED.content,
    modified = EXCLUDED.modified,
    path = EXCLUDED.path,
    ranking = EXCLUDED.ranking;