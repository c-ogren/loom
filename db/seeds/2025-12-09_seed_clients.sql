INSERT INTO clients (client_id, client_secret_hash)
VALUES
  ('demo-client', '$argon2id$v=19$m=19456,t=2,p=1$GTqPYjjc4vTcsnRPcqUBwQ$BOWyu3b74DDSDIjcbP+jpd7eiKYTgNI2i766FE2ul10') -- replace with a real hash
ON DUPLICATE KEY UPDATE client_id = client_id;

-- Fetch inserted client id
-- Note: Adjust if you prefer a single statement. This assumes client exists.
-- For simplicity, reusing SELECT in subsequent inserts.

-- Redirect URIs
INSERT INTO client_redirect_uris (client_id_ref, redirect_uri)
SELECT id, 'http://localhost:3000/callback' FROM clients WHERE client_id = 'demo-client'
ON DUPLICATE KEY UPDATE redirect_uri = redirect_uri;

-- Grant types
INSERT INTO client_grant_types (client_id_ref, grant_type)
SELECT id, 'authorization_code' FROM clients WHERE client_id = 'demo-client'
ON DUPLICATE KEY UPDATE grant_type = grant_type;

INSERT INTO client_grant_types (client_id_ref, grant_type)
SELECT id, 'refresh_token' FROM clients WHERE client_id = 'demo-client'
ON DUPLICATE KEY UPDATE grant_type = grant_type;

-- Scopes
INSERT INTO client_scopes (client_id_ref, scope)
SELECT id, 'openid' FROM clients WHERE client_id = 'demo-client'
ON DUPLICATE KEY UPDATE scope = scope;

INSERT INTO client_scopes (client_id_ref, scope)
SELECT id, 'read' FROM clients WHERE client_id = 'demo-client'
ON DUPLICATE KEY UPDATE scope = scope;