-- Drop sessions table first because it depends on users
DROP TABLE IF EXISTS sessions;

-- Drop session_status type
DROP TYPE IF EXISTS session_status;

-- Drop user_recovery_emails table
DROP TABLE IF EXISTS user_recovery_emails;

-- Drop user_recovery_codes table
DROP TABLE IF EXISTS user_recovery_codes;

-- Drop users table
DROP TABLE IF EXISTS users;

-- Drop functions
DROP FUNCTION IF EXISTS update_modified_column;
DROP FUNCTION IF EXISTS session_table_delete_expired_rows;

-- Drop triggers
DROP TRIGGER IF EXISTS update_users_time ON users;
DROP TRIGGER IF EXISTS update_sessions_time ON sessions;
DROP TRIGGER IF EXISTS delete_expired_sessions ON sessions;

-- Drop indexes
DROP INDEX IF EXISTS idx_users_id;
DROP INDEX IF EXISTS idx_users_email;

DROP INDEX IF EXISTS idx_sessions_user_id;
DROP INDEX IF EXISTS idx_sessions_expires_at;

DROP INDEX IF EXISTS idx_user_recovery_emails_user_id;
DROP INDEX IF EXISTS idx_user_recovery_emails_recovery_email;

DROP INDEX IF EXISTS idx_user_recovery_codes_user_id;
DROP INDEX IF EXISTS idx_user_recovery_codes_recovery_code;