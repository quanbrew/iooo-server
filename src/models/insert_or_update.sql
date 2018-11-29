INSERT INTO items (id, path, content, ranking, expand, created, modified)
VALUES (
  $1, text2ltree($2), $3, $4, $5, TIMESTAMP 'now', TIMESTAMP 'now'
)
ON CONFLICT (id) DO UPDATE
  SET
    content = EXCLUDED.content,
    modified = EXCLUDED.modified,
    path = EXCLUDED.path,
    expand = EXCLUDED.expand,
    ranking = EXCLUDED.ranking;