INSERT INTO items (id, path, content, ranking, expand, deleted, created, modified)
VALUES (
  $1, text2ltree($2), $3, $4, $5, $6, TIMESTAMP 'now', TIMESTAMP 'now'
)
ON CONFLICT (id) DO UPDATE
  SET
    content = EXCLUDED.content,
    modified = EXCLUDED.modified,
    path = EXCLUDED.path,
    expand = EXCLUDED.expand,
    deleted = EXCLUDED.deleted,
    ranking = EXCLUDED.ranking;