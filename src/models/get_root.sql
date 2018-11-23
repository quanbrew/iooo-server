SELECT (id, content, metadata, favorite, tags, created, modified)
FROM items
WHERE path ~ '*{1}';