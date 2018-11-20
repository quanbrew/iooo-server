SELECT ltree2text(path)
FROM items
WHERE id = $1;