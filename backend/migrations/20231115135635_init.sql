-- Add migration script here

CREATE TABLE IF NOT EXISTS log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    message TEXT,
    old_metadata json,
    new_metadata json,
    source_path TEXT NOT NULL,
    target_path TEXT,
    acoustid_score FLOAT,
    retry_count INT NOT NULL
);
