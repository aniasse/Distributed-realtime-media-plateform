CREATE TABLE IF NOT EXISTS rooms (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    room_name VARCHAR(255) NOT NULL,
    room_type VARCHAR(50) NOT NULL,
    max_participants INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    state VARCHAR(20) NOT NULL DEFAULT 'active',
    metadata JSONB
);

CREATE TABLE IF NOT EXISTS peers (
    id UUID PRIMARY KEY,
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    username VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL,
    connected_at TIMESTAMP NOT NULL DEFAULT NOW(),
    bandwidth_estimate JSONB,
    connection_state VARCHAR(20) NOT NULL DEFAULT 'connected',
    ice_candidates JSONB,
    dtls_fingerprints JSONB,
    metadata JSONB
);

CREATE TABLE IF NOT EXISTS tracks (
    id UUID PRIMARY KEY,
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    publisher_id UUID NOT NULL REFERENCES peers(id) ON DELETE CASCADE,
    kind VARCHAR(20) NOT NULL,
    ssrc INTEGER NOT NULL,
    media_info JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS recordings (
    id UUID PRIMARY KEY,
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    recording_name VARCHAR(255) NOT NULL,
    recording_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'started',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    duration_seconds INTEGER,
    file_path VARCHAR(500),
    metadata JSONB
);

CREATE TABLE IF NOT EXISTS streams (
    id UUID PRIMARY KEY,
    stream_key VARCHAR(255) UNIQUE NOT NULL,
    stream_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    resolution JSONB,
    bitrate INTEGER,
    connected_peers INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    metadata JSONB
);

CREATE INDEX idx_rooms_tenant_id ON rooms(tenant_id);
CREATE INDEX idx_peers_room_id ON peers(room_id);
CREATE INDEX idx_tracks_room_id ON tracks(room_id);
CREATE INDEX idx_tracks_publisher_id ON tracks(publisher_id);
CREATE INDEX idx_recordings_room_id ON recordings(room_id);
CREATE INDEX idx_streams_stream_key ON streams(stream_key);

-- Create initial admin user for testing
INSERT INTO users (id, username, email, password_hash) VALUES 
('00000000-0000-0000-0000-000000000001', 'admin', 'admin@example.com', '$2b$12$U6F1yGv8vL0v5t8kL0v5t8kL0v5t8kL0v5t8kL0v5t8kL0v5t8kL0v5t8');

INSERT INTO roles (id, user_id, role_name) VALUES 
('00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'SuperAdmin');