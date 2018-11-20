SELECT
  L.id, R.id, L.content, L.fold, L.metadata, L.deleted,
  L.favorite, L.tags, L.created, L.modified
FROM
  items L LEFT JOIN items R
  ON subpath(L.path, 0, -1) = R.path
ORDER BY (R.id, L.path);