-- minimal reference to a Discord server (aka guild)
CREATE TABLE IF NOT EXISTS guilds (
    id      BIGINT PRIMARY KEY, -- should accomodate Discord snowflakes
    name    VARCHAR(100) NOT NULL,
    icon    TEXT
);

-- minimal reference to a Discord user (which function as Shamebot users)
CREATE TABLE IF NOT EXISTS users (
    id              BIGINT PRIMARY KEY,
    username        VARCHAR(32) NOT NULL,
    discriminator   VARCHAR(4) NOT NULL,
    avatar_hash     TEXT NOT NULL
);

-- many-to-many relationship between users and guilds
CREATE TABLE IF NOT EXISTS user_guild (
    user_id     BIGINT REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    guild_id    BIGINT REFERENCES guilds (id) ON UPDATE CASCADE,
    CONSTRAINT user_guild_pkey PRIMARY KEY (user_id, guild_id)
);

-- user can upload proof that they've completed the task,
-- typically to be used in conjuction w/ an accountability
-- partner
CREATE TABLE IF NOT EXISTS proof (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    content     TEXT,
    image       TEXT,
    approved    BOOLEAN DEFAULT false
);

CREATE TABLE IF NOT EXISTS lists (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    title       VARCHAR(80) NOT NULL,
    user_id     BIGINT REFERENCES users (id) NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid (),
    list_id     uuid REFERENCES lists (id) NOT NULL,
    user_id     BIGINT REFERENCES users (id) NOT NULL,
    title       VARCHAR(80) NOT NULL,
    content     TEXT,
    checked     BOOLEAN DEFAULT false,
    pester      SMALLINT DEFAULT 0,
    due_at      BIGINT DEFAULT 0, -- this is a UNIX timestamp for easy formatting on Discord
    proof_id    uuid REFERENCES proof (id)
);

CREATE TYPE accepted AS ENUM ('accepted', 'pending', 'rejected');

CREATE TABLE IF NOT EXISTS accountability_requests (
    requesting_user     BIGINT REFERENCES users (id) ON DELETE CASCADE,
    requested_user      BIGINT REFERENCES users (id) ON DELETE CASCADE,
    task_id             uuid REFERENCES tasks (id) ON DELETE CASCADE,
    status              accepted DEFAULT 'pending',
    CONSTRAINT accountability_request_pk PRIMARY KEY (requested_user, task_id)
);
