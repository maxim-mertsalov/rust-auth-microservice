default: help

help:
	@echo "Makefile commands:"
	@echo "  install_sqlx  - Install the sqlx-cli tool"
	@echo "  create_db     - Create the database and run migrations"
	@echo "  drop_db       - Drop the database"

install_sqlx:
	@echo "Installing sqlx-cli..."
	cargo install sqlx-cli --no-default-features --features postgres
	@echo "sqlx-cli installed."

create_db:
	@echo "Creating database..."
	sqlx database create
	@echo "Running migrations..."
	sqlx migrate run
	@echo "All done!"

drop_db:
	@echo "Dropping database..."
	sqlx database drop
	@echo "Database dropped."

.PHONY: install_sqlx