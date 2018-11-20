UPDATE items
SET ranking = ranking + 1
WHERE path ~ (text($1))::lquery AND ranking >= $2;