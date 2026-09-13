DROP TABLE IF EXISTS user_watchlist_group_memberships;
DROP TABLE IF EXISTS user_watchlist_groups;

ALTER TABLE user_watchlist
    DROP CONSTRAINT IF EXISTS user_watchlist_ungrouped_position_nonnegative;
ALTER TABLE user_watchlist
    DROP COLUMN IF EXISTS ungrouped_position;
