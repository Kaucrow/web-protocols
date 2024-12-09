<div align="center">
    <h1><b>S</b>end <b>M</b>ail <b>T</b>o <b>P</b>ebbles - SMTP Server</h1>
    It might be best not to interrupt him though!
</div>

<br>

This project is a reliable SMTP server written in Rust, with user registration and authentication, support for file uploads, and secure connections via rustls.

## Table of contents
* [Endpoints](#endpoints)

## Endpoints
**NOTE**: All endpoints may return an `HTTP 500` response, indicating that something went wrong on the server side. Otherwise, they will return any of the responses listed below.

### User Registration
---
* **URL**: `/auth/register`
* **Method**: `POST`
* **Description**: Registers a new user
* **Request body**:
```
{
    email: "napstablook@shadedcitadel.xyz"
    password: "12345678"
    name: "Napstablook"
}
```
* **Response**:
    * Success: `HTTP 200`
    * Email already in use: `HTTP 409`
    ```
    {
        "A user with the provided email already exists"
    }
    ```

### User Login
---
* **URL**: `/auth/login`
* **Method**: `POST`
* **Description**: Logs in a user
* **Request body**:
```
{
    email: "napstablook@shadedcitadel.xyz"
    password: "12345678"
}
```
* **Response**:
    * Success: `HTTP 200`
    * User not found: `HTTP 404`
    ```
    {
        "Could not find a user with the provided email"
    }
    ```

### Send an email
**NOTE**: The sender must be registered on the domain that the app is being served on.
---
* **URL**: `/emails/send`
* **Method**: `POST`
* **Description**: Sends an email using the SMTPebbles server
* **Request body**:
```
{
    sender: "napstablook@shadedcitadel.xyz",
    recipient: "mtt@gmail.com"
    subject: "MTT show"
    body: "Uh...\nHi Mettaton...\nI really liked watching your show...\nMy life is pretty boring, but... seeing you on the screen...\nbrought excitement to my life, vicariously."
}
```
* **Response**:
    * Success: `HTTP 200`
    * The sender is not valid: `HTTP 400`
    * Authentication failed: `HTTP 401`