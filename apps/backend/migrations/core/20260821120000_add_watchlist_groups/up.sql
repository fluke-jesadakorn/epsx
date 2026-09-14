-- Portfolio organizer storage. This migration is deliberately additive: the
-- existing user_watchlist rows remain the source of truth for whether a symbol
-- is watched, while groups only organize those rows.
ALTER TABLE user_watchlist
    ADD COLUMN IF NOT EXISTS ungrouped_position INTEGER;

WITH ranked AS (
    SELECT id,
           ROW_NUMBER() OVER (
               PARTITION BY wallet_address
               ORDER BY added_at ASC, id ASC
           ) - 1 AS position
    FROM user_watchlist
)
UPDATE user_watchlist AS watchlist
SET ungrouped_position = ranked.position
FROM ranked
WHERE watchlist.id = ranked.id
  AND watchlist.ungrouped_position IS NULL;

ALTER TABLE user_watchlist
    ALTER COLUMN ungrouped_position SET NOT NULL;

ALTER TABLE user_watchlist
    DROP CONSTRAINT IF EXISTS user_watchlist_ungrouped_position_nonnegative;
ALTER TABLE user_watchlist
    ADD CONSTRAINT user_watchlist_ungrouped_position_nonnegative
    CHECK (ungrouped_position >= 0);

CREATE TABLE IF NOT EXISTS user_watchlist_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(255) NOT NULL REFERENCES wallet_users(wallet_address) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_watchlist_groups_name_length CHECK (char_length(btrim(name)) BETWEEN 1 AND 50),
    CONSTRAINT user_watchlist_groups_position_nonnegative CHECK (position >= 0),
    UNIQUE (id, wallet_address)
);

CREATE UNIQUE INDEX IF NOT EXISTS user_watchlist_groups_owner_name_unique
    ON user_watchlist_groups(wallet_address, lower(btrim(name)));
CREATE INDEX IF NOT EXISTS user_watchlist_groups_owner_position
    ON user_watchlist_groups(wallet_address, position, id);

CREATE TABLE IF NOT EXISTS user_watchlist_group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL,
    wallet_address VARCHAR(255) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_watchlist_membership_position_nonnegative CHECK (position >= 0),
    CONSTRAINT user_watchlist_membership_group_owner_fk
        FOREIGN KEY (group_id, wallet_address)
        REFERENCES user_watchlist_groups(id, wallet_address)
        ON DELETE CASCADE,
    CONSTRAINT user_watchlist_membership_symbol_owner_fk
        FOREIGN KEY (wallet_address, symbol)
        REFERENCES user_watchlist(wallet_address, symbol)
        ON DELETE CASCADE,
    UNIQUE (group_id, symbol)
);

CREATE INDEX IF NOT EXISTS user_watchlist_memberships_group_position
    ON user_watchlist_group_memberships(group_id, position, id);
CREATE INDEX IF NOT EXISTS user_watchlist_memberships_owner_symbol
    ON user_watchlist_group_memberships(wallet_address, symbol);
