-- Delete duplicate Free Plans from groups table
-- Identify by name or price being 0/null and subscription type
-- Explicitly targeting "Free Plan" name or variants

DELETE FROM groups 
WHERE (name ILIKE '%Free Plan%' OR price IS NULL OR price = 0)
  AND group_type = 'subscription'
  -- Safety check: don't delete plans with price > 0 just in case logic above is loose
  AND (price IS NULL OR price = 0);
