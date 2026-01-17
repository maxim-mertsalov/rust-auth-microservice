# API Documentation
Base URL: `<Server Address>/api/v1`

## General

### `GET /auth`
Get users list

#### Request:
No parameters.

#### Response:
```json
{
    "status": "success",
    "message": "Users fetched successfully",
    "data": [
        {
            "id": "<UUID>",
            "email": "email@example.com",
            "first_name": "Name",
            "last_name": "Lastname"
        }
    ]
}
```

---

### `GET /auth/<user_id>`
Get user by ID

#### Request:
Path parameter `user_id`: UUID of the user.

#### Response:
```json
{
    "status": "success",
    "message": "User fetched successfully",
    "data": {
        "id": "<UUID>",
        "email": "email@example.com",
        "first_name": "Name",
        "last_name": "Lastname",
        "is_verified": true,
        "is_two_factor": true,
        "updated_at": "<DateTime>",
        "created_at": "<DateTime>"
    }
}
```

---

## Session Management

### `POST /auth/refresh`
Refresh access token using a pair of access and refresh tokens.

#### Request:
```json
{
    "device_info": {
        "ip_address": "<User IP Address>",
        "user_agent": "<User Agent String>"
    },
    "access_token": "<access_token>",
    "refresh_token": "<refresh_token>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Token refreshed successfully!",
    "data": {
        "access_token": "<new_access_token>",
        "refresh_token": "<new_refresh_token>"
    }
}
```

---

### `POST /auth/check`
Check if access token is valid.

#### Request:
```json
{
    "access_token": "<access_token>"
}
```

#### Response:
```json
{
    "status": "error",
    "message": "Access token is expired!",
    "errors": null
}
```

---

### `DELETE /auth/logout`
Logout from current sessions

#### Request:
```json
{
    "access_token": "<access_token>",
    "refresh_token": "<refresh_token>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "User logged out successfully!",
    "data": null
}
```

---

### `POST /auth/sessions`
Get all active sessions for the user.

#### Request:
```json
{
    "access_token": "<access_token>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "All sessions got successfully",
    "data": [
        {
            "id": "<UUID>", 
            "device_info": {
                "ip_address": "<IP Address>", 
                "user_agent": "<User Agent String>"
            },
            "status": "ACTIVE",
            "expires_at": "<DateTime>",
            "updated_at": "<DateTime>",
            "created_at": "<DateTime>"
        }
    ]
}
```

---

### `DELETE /auth/sessions/`
Logout from specific session by ID

#### Request:
```json
{
    "access_token": "<access_token>",
    "session_id": "<session_id>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Session terminated successfully!",
    "data": null
}
```

---

### `DELETE /auth/sessions/all`
Logout from all active sessions besides the current one.

#### Request:
```json
{
    "access_token": "<access_token>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "All other sessions terminated successfully!",
    "data": null
}
```

---

## Sign Up

### `POST /auth/signup/init`
Initiate the sign-up process by providing a profile data.

#### Request:
```json
{
    "device_info": {
        "ip_address": "<User IP Address>",
        "user_agent": "<User Agent String>"
    },
    "scopes": ["full_access", "email", "profile", "openid", "offline_access"],
    "final_redirect_url": "/me",

    "first_name": "Name",
    "last_name": "Lastname"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Sign-up session successfully created!",
    "data": {
        "session_token": "<session_token>",
        "next_stage": "EmailStage"
    }
}
```

---

### `POST /auth/signup/email`
Submit email for verification.

#### Request:
```json
{
    "session_token": "<session_token>",
    "email": "email@example.com"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Email set successfully",
    "data": {
        "next_stage": "EmailVerificationStage"
    }
}
```

---

### `POST /auth/signup/email/resend`
Resend email verification code.

#### Request:
```json
{
    "session_token": "<session_token>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Email code resent successfully",
    "data": null
}
```

---

### `POST /auth/signup/email/verify`
Verify email with the code sent to the provided email address.

#### Request:
```json
{
    "session_token": "<session_token>",
    "verification_code": "123456"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Email verified successfully",
    "data": {
        "next_stage": "PasswordStage"
    }
}
```

---

### `POST /auth/signup/password`
Set password for the new account.

#### Request:
```json
{
    "session_token": "<session_token>",
    "password": "StrongPassword123!"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Password set successfully",
    "data": {
        "next_stage": "Redirect"
    }
}
```

---

### `POST /auth/signup/back`
Go back to the previous stage in the sign-up flow.

#### Request:
```json
{
    "session_token": "<session_token>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "You have returned successfully",
    "data": {
        "next_stage": "PasswordStage"
    }
}
```

---

### `POST /auth/signup/final`
Finalize the sign-up process and receive tokens.

#### Request:
```json
{
    "session_token": "<session_token>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "User created successfully",
    "data": {
        "redirect_url": "/me",
        "access_token": "<access_token>",
        "refresh_token": "<refresh_token>",
        "email": "email@example.com",
        "first_name": "Name",
        "last_name": "Lastname"
    }
}
```

---

## Sign In

### `POST /auth/signin/init`
Initiate the sign-in process by providing email.

#### Request:
```json
{
    "device_info": {
        "ip_address": "<User IP Address>",
        "user_agent": "<User Agent String>"
    },
    "scopes": ["full_access", "email", "profile", "openid", "offline_access"],
    "final_redirect_url": "/me",

    "email": "email@example.com"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Sign-in session successfully created!",
    "data": {
        "session_token": "<session_token>",
        "next_stage": "SetPasswordStage"
    }
}
```

---

### `POST /auth/signin/password`
Submit password for authentication.

#### Request:
```json
{
    "session_token": "<session_token>",
    "password": "StrongPassword123!"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Password verified successfully",
    "data": {
        "next_stage": "Redirect"
    }
}
```

---

### `POST /auth/signin/final`
Finalize the sign-in process and receive tokens.
#### Request:
```json
{
    "session_token": "<session_token>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Email is verified successfully",
    "data": {
        "redirect_url": "/me",
        "access_token": "<access_token>",
        "refresh_token": "<refresh_token>",
        "email": "email@example.com",
        "first_name": "Name",
        "last_name": "Lastname"
    }
}
```

---

### `POST /auth/signin/email/resend`
Resend email verification code during sign-in 2FA authentication stage.

#### Request:
```json
{
    "session_token": "<session_token>"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Email code resent successfully",
    "data": null
}
```

---

### `POST /auth/signin/email/verify`
Verify email with the code sent to the provided email address during sign-in 2FA authentication stage.

#### Request:
```json
{
    "session_token": "<session_token>",
    "verification_code": "123456"
}
```

#### Response:
```json
{
    "status": "success",
    "message": "Email is verified successfully",
    "data": {
        "next_stage": "Redirect"
    }
}
```

---