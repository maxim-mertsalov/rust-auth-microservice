# Rust Authentication Microservice
A high-performance, results-driven authentication service built with Rust and Actix Web. This service implements a secure session management system using JWT rotation and a multi-stage state machine for user onboarding and authentication.

## TODO feature list
### Core Authentication
- [x] Sign-Up Flow
  - [x] Plain password registration
  - [x] Email verification system

- [ ] Sign-In Flow
  - [x] Plain password authentication
  - [x] Email 2FA (TOTP/Verification)
  - [ ] Recovery codes integration
  - [ ] Multi-email support (Alternative login aliases)
  - [ ] Passkey (WebAuthn) support

- [ ] External Providers (OAuth2/OIDC)
  - [ ] Google/GitHub integration
  - [ ] Smart flow switching (Automatic Sign-In vs. Sign-Up)
  - [ ] Account linking (Merge OAuth with existing email accounts)

### Session & Security
- [x] Session Management
  - [x] Persistent session storage (Redis)
  - [x] Refresh token rotation (Security best practice)
  - [x] Device metadata tracking (User-Agent/IP)
  - [x] Session control: List active, Revoke specific, and Global logout

- [ ] Account Recovery
  - [ ] Password reset flow (Identity-verified)
  - [ ] 2FA Recovery code generation

### User Management
- [ ] Profile & Security Settings
  - [x] Basic profile updates (Name, etc.)
  - [ ] Email change process (Requires new verification)
  - [ ] Account deletion/deactivation flow


## Architecture & Logic
The service is engineered around a **State-Machine Lifecycle**, ensuring that authentication and registration are not just endpoints, but controlled processes.

- **Deterministic Transitions**: Each step in the `Sign-In` or `Sign-Up` flow requires a valid `session_token`. This prevents users from skipping steps (e.g., setting a password before email verification).

- **Performance-First Storage**: By utilizing **Redis** for session management and **PostgreSQL** for persistent data, the service offloads high-frequency token checks to in-memory storage, significantly reducing primary database load.

- **Asynchronous Runtime**: Leveraging Actix Web’s actor-based model to handle thousands of concurrent authentication requests with minimal resource overhead.


## Core Features

### Security & Token Management
- **JWT Rotation (Refresh/Access)**: Implements a robust "Refresh Token" strategy to minimize security risks. When an access token expires, a new pair is issued, invalidating the old refresh token to prevent reuse.

- **Secure Password Hashing**: Utilizes evidence-based hashing (Argon2/Bcrypt) to ensure user credentials are encrypted and resistant to brute-force attacks.

- **Token Scoping**: Supports granular access control through `scopes`, allowing you to restrict what specific tokens can do within your ecosystem.

### Session & Device Control
- **Real-time Session Auditing**: Users can retrieve a list of all active sessions, including IP addresses and User-Agent strings, to identify unauthorized access.

- **Remote Revocation**: Includes the ability to terminate specific sessions by ID or perform a "Global Logout," which immediately wipes all active Redis keys for that user.

- **Device Awareness**: Tracks metadata for every login event, providing a foundation for security notifications and suspicious activity detection.

### Developer Experience
- **Low-Friction Structure**: The project follows a strictly organized directory layout, separating concerns between API handlers, core logic, and database repositories.

- **Standardized API Responses**: Clean, predictable JSON payloads for all state transitions and error handling.


## Project Structure
```text
src/
├── api/            # API route handlers
├── config/         # Configuration management
├── db/             # Database connections
├── dto/            # Data Transfer Objects
├── errors/         # Custom error handling
├── models/         # Data models
├── repositories/   # Data access layer
├── services/       # Business logic
├── state/          # Application state
├── utils/          # Utility functions
└── main.rs         # Application entry point
```

## Deployment & Setup

### From Scratch

1. **Clone the repository**: 
   ```bash
   git clone https://github.com/maxim-mertsalov/rust-auth-microservice
   cd rust-auth-microservice
   ```

2. **Configure environment variables**: Create a `.env` file based on `.env.example` and set your configuration values.

3. **Set up PostgreSQL and Redis**: Ensure you have PostgreSQL and Redis. 
    - You can create postgres database using Makefile

4. **Build and run the application**:
   ```bash
   cargo run --release
   ```

### Using Docker Compose
1. **Clone the repository**: 
```bash
   git clone https://github.com/maxim-mertsalov/rust-auth-microservice
   cd rust-auth-microservice
  ```
2. **Create a `.env` file**: Based on `.env.example`, set your configuration values.

3. **Start services with Docker Compose**:
```bash
   docker-compose up --build
  ```


## API Endpoints 
Detailed API documentation can be found in the [API.md](API.md) file.

