-- Add migration script here

-- Clients table
CREATE TABLE IF NOT EXISTS clients (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  client_id VARCHAR(255) NOT NULL UNIQUE,
  client_secret_hash VARCHAR(255) NOT NULL, -- store a hash (e.g., argon2id or bcrypt)
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NULL DEFAULT NULL ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Redirect URIs (one-to-many)
CREATE TABLE IF NOT EXISTS client_redirect_uris (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  client_id_ref BIGINT UNSIGNED NOT NULL,
  redirect_uri TEXT NOT NULL,
  PRIMARY KEY (id),
  CONSTRAINT fk_client_redirect_uris_client
    FOREIGN KEY (client_id_ref) REFERENCES clients(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
  INDEX idx_client_redirect_uris_client (client_id_ref)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Grant types (authorization_code, refresh_token, client_credentials, etc.)
CREATE TABLE IF NOT EXISTS client_grant_types (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  client_id_ref BIGINT UNSIGNED NOT NULL,
  grant_type VARCHAR(64) NOT NULL,
  PRIMARY KEY (id),
  CONSTRAINT fk_client_grant_types_client
    FOREIGN KEY (client_id_ref) REFERENCES clients(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
  UNIQUE KEY uniq_client_grant (client_id_ref, grant_type),
  INDEX idx_client_grant_types_client (client_id_ref)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Scopes (space-delimited in OAuth, stored as individual rows)
CREATE TABLE IF NOT EXISTS client_scopes (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  client_id_ref BIGINT UNSIGNED NOT NULL,
  scope VARCHAR(128) NOT NULL,
  PRIMARY KEY (id),
  CONSTRAINT fk_client_scopes_client
    FOREIGN KEY (client_id_ref) REFERENCES clients(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
  UNIQUE KEY uniq_client_scope (client_id_ref, scope),
  INDEX idx_client_scopes_client (client_id_ref)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;