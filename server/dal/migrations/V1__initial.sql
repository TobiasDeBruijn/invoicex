CREATE TABLE users (
    id VARCHAR(32) NOT NULL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL
);

CREATE TABLE user_email_verification_tokens (
    token VARCHAR(32) NOT NULL PRIMARY KEY,
    email VARCHAR(64) NOT NULL,
    user_id VARCHAR(32) NOT NULL,
    expires_at BIGINT NOT NULL
);

CREATE TABLE user_emails (
    email VARCHAR(64) NOT NULL PRIMARY KEY,
    user_id VARCHAR(32) NOT NULL,
    verified BOOL DEFAULT FALSE
);

CREATE TABLE user_passwords (
    id VARCHAR(32) NOT NULL PRIMARY KEY,
    hash TEXT NOT NULL
);

CREATE TABLE user_authentication_methods (
    id VARCHAR(32) NOT NULL PRIMARY KEY,
    method VARCHAR(32) NOT NULL
);

CREATE TABLE user_sessions (
    id VARCHAR(64) NOT NULL PRIMARY KEY,
    user_id VARCHAR(32) NOT NULL,
    last_used BIGINT NOT NULL,
    expires_at BIGINT NOT NULL
);

CREATE TABLE service_tokens (
    token VARCHAR(64) NOT NULL PRIMARY KEY,
    associated_user_id VARCHAR(32) NOT NULL
);

CREATE TABLE orgs (
    id VARCHAR(32) NOT NULL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE TABLE org_user_links (
    org_id VARCHAR(32) NOT NULL,
    user_id VARCHAR(32) NOT NULL,
    org_admin BOOL NOT NULL DEFAULT FALSE,
    PRIMARY KEY (org_id, user_id)
);

CREATE TABLE org_user_link_scopes (
    org_id VARCHAR(32) NOT NULL,
    user_id VARCHAR(32) NOT NULL,
    scope_name VARCHAR(64) NOT NULL,
    PRIMARY KEY (org_id, user_id)
);