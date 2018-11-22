SELECT
  L.id, R.id, L.content, L.fold, L.metadata,
  L.favorite, L.tags, L.created, L.modified
FROM
  items L LEFT JOIN items R
  ON subpath(L.path, 0, -1) = R.path
WHERE NOT L.deleted
ORDER BY (R.id, L.ranking);