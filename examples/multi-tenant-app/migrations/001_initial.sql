-- Initial schema for multi-tenant application
-- Each tenant gets their own copy of this schema in their isolated database

-- Items table for storing tenant-specific data
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Index for faster queries
CREATE INDEX IF NOT EXISTS idx_items_created_at ON items(created_at DESC);

-- Tenant metadata
CREATE TABLE IF NOT EXISTS tenant_info (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    item_count INTEGER DEFAULT 0,
    last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Trigger to update the updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_items_timestamp 
AFTER UPDATE ON items
BEGIN
    UPDATE items SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Trigger to update last_activity in tenant_info
CREATE TRIGGER IF NOT EXISTS update_tenant_activity 
AFTER INSERT OR UPDATE ON items
BEGIN
    UPDATE tenant_info SET last_activity = CURRENT_TIMESTAMP WHERE id = 1;
END;