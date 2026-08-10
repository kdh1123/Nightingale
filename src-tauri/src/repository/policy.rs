//! Versioned detection policy storage. Saving always appends a new version so an
//! operator can see how the policy changed over time.

use super::open_connection;
use crate::domain::policy::SecurityPolicy;
use rusqlite::{params, Connection, OptionalExtension};

pub(super) fn detection_policy_from_connection(
    connection: &Connection,
) -> Result<SecurityPolicy, String> {
    let raw = connection
        .query_row(
            "SELECT policy_json FROM security_policies ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    raw.map(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
        .transpose()
        .map(|policy| policy.unwrap_or_default())
}

pub fn detection_policy(database_path: &std::path::Path) -> Result<SecurityPolicy, String> {
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    detection_policy_from_connection(&connection)
}

pub fn save_detection_policy(
    database_path: &std::path::Path,
    mut policy: SecurityPolicy,
) -> Result<SecurityPolicy, String> {
    policy.validate().map_err(str::to_string)?;
    let connection = open_connection(database_path).map_err(|error| error.to_string())?;
    let next_version = connection
        .query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM security_policies",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    policy.version = next_version;
    connection
        .execute(
            "INSERT INTO security_policies (version, policy_json) VALUES (?1, ?2)",
            params![
                policy.version,
                serde_json::to_string(&policy).map_err(|error| error.to_string())?
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(policy)
}
