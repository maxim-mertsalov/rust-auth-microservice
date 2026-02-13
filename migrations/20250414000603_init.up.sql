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
    DELETE FROM sessions WHERE expires_at < now() OR (status != 'ACTIVE' AND updated_at < now() - interval '7 days');
    RETURN NEW;
END;
$$ language 'plpgsql';


-- Create users table
CREATE TABLE users (
    id                          UUID PRIMARY KEY DEFAULT uuidv7(),
    email                       TEXT UNIQUE NOT NULL,
    password                    TEXT NOT NULL,
    first_name                  TEXT NOT NULL,
    last_name                   TEXT NOT NULL,
    is_verified                 BOOLEAN NOT NULL DEFAULT FALSE,
    is_two_factor               BOOLEAN NOT NULL DEFAULT FALSE,
    days_to_inactive            INT NOT NULL DEFAULT 30,
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);


-- Create status type for sessions
CREATE TYPE session_status AS ENUM ('ACTIVE', 'TERMINATED_BY_USER', 'TERMINATED_BY_ADMIN', 'COMPROMISED', 'EXPIRED');

-- Create sessions table
CREATE TABLE sessions (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id         UUID NOT NULL,
    refresh_token   TEXT NOT NULL,
    device_info     JSONB NOT NULL,
    status          session_status NOT NULL DEFAULT 'ACTIVE',
    expires_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE user_recovery_emails (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id         UUID NOT NULL,
    recovery_email  TEXT UNIQUE NOT NULL,
    verified_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);


CREATE TABLE user_recovery_codes (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id         UUID NOT NULL,
    prefix          TEXT NOT NULL,
    recovery_code   TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);


-- Indexes
CREATE INDEX idx_users_id ON users(id);
CREATE INDEX idx_users_email ON users(email);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

CREATE INDEX idx_user_recovery_emails_user_id ON user_recovery_emails(user_id);
CREATE INDEX idx_user_recovery_emails_recovery_email ON user_recovery_emails(recovery_email)
WHERE verified_at IS NOT NULL;

CREATE INDEX idx_user_recovery_codes_user_id ON user_recovery_codes(user_id);
CREATE INDEX idx_user_recovery_codes_recovery_code ON user_recovery_codes(recovery_code);



-- Create triggers to update updated_at column on modification
CREATE TRIGGER update_users_time
    BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE update_modified_column();

CREATE TRIGGER update_sessions_time
    BEFORE UPDATE ON sessions FOR EACH ROW EXECUTE PROCEDURE update_modified_column();

CREATE TRIGGER delete_expired_sessions
    AFTER INSERT ON sessions EXECUTE PROCEDURE session_table_delete_expired_rows();
