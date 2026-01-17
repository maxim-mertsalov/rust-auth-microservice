-- Create function to update updated_at column
CREATE OR REPLACE FUNCTION update_modified_column()
    RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';


-- Create function to delete expired sessions
CREATE OR REPLACE FUNCTION session_table_delete_expired_rows()
    RETURNS TRIGGER AS
$$
BEGIN
    DELETE FROM sessions WHERE expires_at < now() OR (status != 'ACTIVE' AND updated_at < now() - interval '3 days');
    RETURN NEW;
END;
$$ language 'plpgsql';


-- Create users table
CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                       email TEXT UNIQUE NOT NULL,
                       password TEXT NOT NULL,
                       first_name TEXT NOT NULL,
                       last_name TEXT NOT NULL,
                       is_verified BOOLEAN NOT NULL DEFAULT FALSE,
                       is_two_factor BOOLEAN NOT NULL DEFAULT FALSE,
                       days_to_inactive INT NOT NULL DEFAULT 30,
                       updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- Create status type for sessions
CREATE TYPE session_status AS ENUM ('ACTIVE', 'TERMINATED_BY_USER', 'TERMINATED_BY_ADMIN', 'COMPROMISED', 'EXPIRED');

-- Create sessions table
CREATE TABLE sessions (
                          id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          user_id UUID NOT NULL,
                          refresh_token TEXT NOT NULL,
                          device_info JSONB NOT NULL,
                          status session_status NOT NULL DEFAULT 'ACTIVE',
                          expires_at TIMESTAMPTZ NOT NULL,
                          updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                          created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                          FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX idx_users_id ON users(id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_refresh_token ON sessions(refresh_token);


-- Create triggers to update updated_at column on modification
CREATE TRIGGER update_users_time
    BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE update_modified_column();

CREATE TRIGGER update_sessions_time
    BEFORE UPDATE ON sessions FOR EACH ROW EXECUTE PROCEDURE update_modified_column();

CREATE TRIGGER delete_expired_sessions
    AFTER INSERT ON sessions EXECUTE PROCEDURE session_table_delete_expired_rows();
