CREATE TABLE IF NOT EXISTS users (
    email VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS sent_emails (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_email VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    content VARCHAR(255),
    destination_email VARCHAR(255) NOT NULL,
    destination_name VARCHAR(255) NOT NULL,

    FOREIGN KEY (sender_email) REFERENCES users(email)
);

CREATE TABLE IF NOT EXISTS received_emails (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    receiver_email VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    content VARCHAR(255),
    sender_email VARCHAR(255) NOT NULL,
    sender_name VARCHAR(255) NOT NULL,

    FOREIGN KEY (receiver_email) REFERENCES users(email)
);

CREATE TABLE IF NOT EXISTS sent_emails_files (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sent_email_uuid UUID NOT NULL,
    path VARCHAR(255) NOT NULL,

    FOREIGN KEY (sent_email_uuid) REFERENCES sent_emails(uuid)
);

CREATE TABLE IF NOT EXISTS received_emails_files (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    received_email_uuid UUID NOT NULL,
    path VARCHAR(255) NOT NULL,

    FOREIGN KEY (received_email_uuid) REFERENCES received_emails(uuid)
);

CREATE INDEX IF NOT EXISTS users_email_passwd_idx ON users (email, password);

CREATE INDEX IF NOT EXISTS sender_email_idx ON sent_emails (sender_email);

CREATE INDEX IF NOT EXISTS received_email_idx ON received_emails (receiver_email);

CREATE INDEX IF NOT EXISTS sent_emails_files_idx ON sent_emails_files (sent_email_uuid);

CREATE INDEX IF NOT EXISTS received_emails_files_idx ON received_emails_files (received_email_uuid);