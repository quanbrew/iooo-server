BEGIN;
CREATE EXTENSION IF NOT EXISTS ltree;


CREATE TABLE items (
    id uuid PRIMARY KEY,
    path ltree NOT NULL, -- [...].[parent position].[self position]
    -- owner uuid NOT NULL,
    -- writers uuid[] DEFAULT '{}' NOT NULL,
    content text NOT NULL,
    expand boolean DEFAULT FALSE NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}',
    deleted boolean DEFAULT FALSE NOT NULL,
    ranking integer NOT NULL,
    tags text[] DEFAULT '{}' NOT NULL,
    created timestamp NOT NULL,
    modified timestamp NOT NULL
);

-- CREATE INDEX item_owner ON items (owner);
CREATE INDEX item_tags ON items USING GIN (tags);
-- ltree index https://www.postgresql.org/docs/11/ltree.html#id-1.11.7.30.6
CREATE INDEX item_path ON items USING GIST (path);



-- CREATE TABLE users (
--     id uuid PRIMARY KEY,
--     display_name text NOT NULL,
--     email text UNIQUE NOT NULL,
--     password text NOT NULL, -- salt:hashed
--     joined timestamp NOT NULL,
--     deleted boolean DEFAULT FALSE NOT NULL
-- );

-- CREATE UNIQUE INDEX user_email ON users (email);
COMMIT;