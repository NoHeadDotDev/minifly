-- Create apps table
CREATE TABLE IF NOT EXISTS apps (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    organization_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Create machines table
CREATE TABLE IF NOT EXISTS machines (
    id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL,
    name TEXT NOT NULL,
    state TEXT NOT NULL,
    region TEXT NOT NULL,
    image TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    private_ip TEXT NOT NULL,
    config JSON NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (app_id) REFERENCES apps(id)
);

-- Create volumes table
CREATE TABLE IF NOT EXISTS volumes (
    id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL,
    name TEXT NOT NULL,
    state TEXT NOT NULL,
    size_gb INTEGER NOT NULL,
    region TEXT NOT NULL,
    zone TEXT NOT NULL,
    encrypted BOOLEAN NOT NULL,
    attached_machine_id TEXT,
    created_at TIMESTAMP NOT NULL,
    FOREIGN KEY (app_id) REFERENCES apps(id),
    FOREIGN KEY (attached_machine_id) REFERENCES machines(id)
);

-- Create leases table
CREATE TABLE IF NOT EXISTS leases (
    machine_id TEXT PRIMARY KEY,
    nonce TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    owner TEXT NOT NULL,
    description TEXT,
    version TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    FOREIGN KEY (machine_id) REFERENCES machines(id)
);

-- Create machine_events table
CREATE TABLE IF NOT EXISTS machine_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    machine_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY (machine_id) REFERENCES machines(id)
);

-- Create metadata table
CREATE TABLE IF NOT EXISTS machine_metadata (
    machine_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (machine_id, key),
    FOREIGN KEY (machine_id) REFERENCES machines(id)
);

-- Create indexes
CREATE INDEX idx_machines_app_id ON machines(app_id);
CREATE INDEX idx_volumes_app_id ON volumes(app_id);
CREATE INDEX idx_machine_events_machine_id ON machine_events(machine_id);
CREATE INDEX idx_leases_expires_at ON leases(expires_at);