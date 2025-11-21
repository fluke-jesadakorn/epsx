-- Migration: Add Route Permissions (DOWN)
-- Description: Removes route_permissions table and all data
-- Created: 2025-11-18

-- Drop the route_permissions table and all its data
DROP TABLE IF EXISTS route_permissions;