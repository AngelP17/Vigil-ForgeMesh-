-- Migration: Add incident ordering support
-- This enables drag-and-drop reordering in the incident queue

-- Add rank column for incident ordering (lower = higher priority)
ALTER TABLE incidents ADD COLUMN rank INTEGER DEFAULT 0;

-- Add index for efficient ordering queries
CREATE INDEX IF NOT EXISTS idx_incidents_rank ON incidents(rank);

-- Add index for status + rank queries (queue by status)
CREATE INDEX IF NOT EXISTS idx_incidents_status_rank ON incidents(status, rank);

-- Initialize rank based on existing order (opened_at DESC, severity)
UPDATE incidents SET rank = (
    SELECT COALESCE(MAX(rank), 0) + 1
    FROM incidents AS i2
    WHERE i2.opened_at > incidents.opened_at
    OR (i2.opened_at = incidents.opened_at AND i2.severity > incidents.severity)
);
