use serde_json::from_str;
use sqlx::{MySql, Pool};

use crate::services::{AuthorizeInput, TokenInput};

// TODO: clean this up later
#[allow(dead_code)]
#[derive(Debug)]
pub struct Client {
    pub id: u64,
    pub name: String,
    pub secret_hash: String,
    pub scopes: Option<Vec<String>>,
    pub redirect_uris: Option<Vec<String>>,
}

pub async fn create_client(
    pool: &Pool<MySql>,
    client_id: &str,
    client_secret_hash: &str,
    redirect_uris: &[String],
    grant_types: &[String],
    scopes: &[String],
) -> sqlx::Result<u64> {
    let result = sqlx::query!(
        r#"
        INSERT INTO clients (client_id, client_secret_hash)
        VALUES (?, ?)
        "#,
        client_id,
        client_secret_hash
    )
    .execute(pool)
    .await?;

    // insert into client_redirect_uris
    for uri in redirect_uris {
        sqlx::query!(
            r#"
            INSERT INTO client_redirect_uris (client_id_ref, redirect_uri)
            VALUES (
                ?,
                ?
            )
            "#,
            result.last_insert_id(),
            uri
        )
        .execute(pool)
        .await?;
    }

    // insert into client_scopes
    for scope in scopes {
        sqlx::query!(
            r#"
            INSERT INTO client_scopes (client_id_ref, scope)
            VALUES (
                ?,
                ?
            )
            "#,
            result.last_insert_id(),
            scope
        )
        .execute(pool)
        .await?;
    }

    // insert into client_grant_types
    for grant_type in grant_types {
        sqlx::query!(
            r#"
            INSERT INTO client_grant_types (client_id_ref, grant_type)
            VALUES (
                ?,
                ?
            )
            "#,
            result.last_insert_id(),
            grant_type
        )
        .execute(pool)
        .await?;
    }

    Ok(result.last_insert_id())
}

pub async fn get_by_client_id(
    pool: &Pool<MySql>,
    authorize_input: &AuthorizeInput,
) -> sqlx::Result<Option<Client>> {
    let row = sqlx::query!(
        r#"
        SELECT
          c.id AS `id!: u64`,
          c.client_id AS `name!`,
          c.client_secret_hash AS `secret_hash!`,
          CAST(COALESCE(JSON_ARRAYAGG(cs.scope), JSON_ARRAY()) AS CHAR) AS `scopes_json!`,
          CAST(COALESCE(JSON_ARRAYAGG(cru.redirect_uri), JSON_ARRAY()) AS CHAR) AS `redirect_uris_json!`
        FROM clients c
        LEFT JOIN client_scopes cs ON cs.client_id_ref = c.id
        LEFT JOIN client_redirect_uris cru ON cru.client_id_ref = c.id
        WHERE c.client_id = ?
        AND cru.redirect_uri = ?
        AND cs.scope IN (?)
        GROUP BY c.id, c.client_id, c.client_secret_hash
        "#,
        authorize_input.client_id,
        authorize_input.redirect_uri,
        authorize_input.scopes.join(",")
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Client {
        id: r.id,
        name: r.name,
        secret_hash: r.secret_hash,
        scopes: Some(from_str::<Vec<String>>(&r.scopes_json).unwrap_or_default()),
        redirect_uris: Some(from_str::<Vec<String>>(&r.redirect_uris_json).unwrap_or_default()),
    }))
}

pub async fn get_by_client_token(
    pool: &Pool<MySql>,
    token_input: &TokenInput,
) -> sqlx::Result<Option<Client>> {
    let row = sqlx::query!(
        r#"
        SELECT
          c.id AS `id!: u64`,
          c.client_id AS `name!`,
          c.client_secret_hash AS `secret_hash!`,
          CAST(COALESCE(JSON_ARRAYAGG(cs.scope), JSON_ARRAY()) AS CHAR) AS `scopes_json!`,
          CAST(COALESCE(JSON_ARRAYAGG(cru.redirect_uri), JSON_ARRAY()) AS CHAR) AS `redirect_uris_json!`
        FROM clients c
        LEFT JOIN client_scopes cs ON cs.client_id_ref = c.id
        LEFT JOIN client_redirect_uris cru ON cru.client_id_ref = c.id
        WHERE c.client_id = ?
        AND cru.redirect_uri = ?
        GROUP BY c.id, c.client_id, c.client_secret_hash
        "#,
        token_input.client_id,
        token_input.redirect_uri,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Client {
        id: r.id,
        name: r.name,
        secret_hash: r.secret_hash,
        scopes: None,
        redirect_uris: None,
    }))
}
