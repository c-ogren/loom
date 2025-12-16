/*
 * SETEX variant:
 * Usage: redis_set_ex!(conn, "auth_code", code, payload, AUTH_CODE_EXPIRATION_SECS);
 */

#[macro_export]
macro_rules! redis_set_ex {
    ($conn:expr, $prefix:expr, $key:expr, $value:expr, $ttl:expr) => {{
        use redis::AsyncCommands;
        let full_key = format!("{}:{}", $prefix, $key);
        // annotate unit to avoid never-type fallback warnings
        let _: () = $conn.set_ex(full_key, $value, $ttl).await?;
    }};
}

/*
 * GETDEL variant:
 * Usage: let val: Option<String> = redis_getdel!(conn, "auth_code", code)?;
 */

#[macro_export]
macro_rules! redis_getdel {
    ($conn:expr, $prefix:expr, $key:expr) => {{
        use redis::AsyncCommands;
        let full_key = format!("{}:{}", $prefix, $key);
        let val = $conn.get_del(full_key).await?;
        val
    }};
}

/*
 * GET variant:
 * Usage: let val: Option<String> = redis_get!(conn, "auth_code", code)?;
 */

#[macro_export]
macro_rules! redis_get {
    ($conn:expr, $prefix:expr, $key:expr) => {{
        use redis::AsyncCommands;
        let full_key = format!("{}:{}", $prefix, $key);
        let val = $conn.get(full_key).await?;
        val
    }};
}
