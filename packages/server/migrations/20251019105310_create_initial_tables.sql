-- Tabelle für alle bekannten Agents
CREATE TABLE agents (
    id TEXT PRIMARY KEY NOT NULL,
    hostname TEXT NOT NULL,
    -- 'online', 'offline', 'busy'
    status TEXT NOT NULL DEFAULT 'offline',
    last_heartbeat INTEGER NOT NULL
);

-- Tabelle für Jobs, die ausgeführt werden sollen
CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    agent_id TEXT, -- Welcher Agent hat den Job?
    -- 'pending', 'running', 'success', 'failed'
    status TEXT NOT NULL DEFAULT 'pending',
    repository_url TEXT NOT NULL,
    commands TEXT NOT NULL, -- Als JSON-Array gespeichert
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY(agent_id) REFERENCES agents(id)
);