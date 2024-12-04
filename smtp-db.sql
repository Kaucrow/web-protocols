CREATE TABLE User (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL
);

CREATE TABLE SentEmails (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sender_uuid UUID NOT NULL,
    subject VARCHAR(255) NOT NULL,
    content VARCHAR(255),
    destination_email VARCHAR(255) NOT NULL,
    destination_name VARCHAR(255) NOT NULL,

    FOREIGN KEY (sender_id) REFERENCES User(uuid)
);

CREATE TABLE ReceivedEmails (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    receiver_uuid UUID NOT NULL,
    subject VARCHAR(255) NOT NULL,
    content VARCHAR(255),
    sender_email VARCHAR(255) NOT NULL,
    sender_name VARCHAR(255) NOT NULL,

    FOREIGN KEY (receiver_id) REFERENCES User(uuid)
);

CREATE TABLE SentEmailFiles (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sent_email_uuid UUID NOT NULL,
    path VARCHAR(255) NOT NULL,

    FOREIGN KEY (sent_email_uuid) REFERENCES SentEmails(uuid)
);

CREATE TABLE ReceivedEmailFiles (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    received_email_uuid UUID NOT NULL,
    path VARCHAR(255) NOT NULL,

    FOREIGN KEY (received_email_uuid) REFERENCES ReceivedEmails(uuid)
);