use bitwarden::secrets_manager::access_policies::{AccessPolicyEntry, GrantedProjectEntry};
use color_eyre::eyre::{Result, bail};
use uuid::Uuid;

/// Parse "read" | "write" | "manage" into (read, write, manage) booleans.
pub(crate) fn parse_permission(s: &str) -> Result<(bool, bool, bool)> {
    match s.to_lowercase().as_str() {
        "read" => Ok((true, false, false)),
        "write" => Ok((true, true, false)),
        "manage" => Ok((true, true, true)),
        _ => bail!("Invalid permission '{}'. Expected: read, write, or manage", s),
    }
}

/// Parse repeated flag pairs like ["<uuid>", "manage", "<uuid>", "read"]
/// into Vec<AccessPolicyEntry>.
pub(crate) fn parse_policy_flags(pairs: &[String]) -> Result<Vec<AccessPolicyEntry>> {
    pairs
        .chunks(2)
        .map(|chunk| {
            let grantee_id = Uuid::parse_str(&chunk[0])
                .map_err(|_| color_eyre::eyre::eyre!("Invalid UUID '{}'", chunk[0]))?;
            let (read, write, manage) = parse_permission(&chunk[1])?;
            Ok(AccessPolicyEntry {
                grantee_id,
                read,
                write,
                manage,
            })
        })
        .collect()
}

/// Parse repeated project flag pairs into Vec<GrantedProjectEntry>.
pub(crate) fn parse_granted_project_flags(pairs: &[String]) -> Result<Vec<GrantedProjectEntry>> {
    pairs
        .chunks(2)
        .map(|chunk| {
            let project_id = Uuid::parse_str(&chunk[0])
                .map_err(|_| color_eyre::eyre::eyre!("Invalid UUID '{}'", chunk[0]))?;
            let (read, write, manage) = parse_permission(&chunk[1])?;
            Ok(GrantedProjectEntry {
                project_id,
                read,
                write,
                manage,
            })
        })
        .collect()
}

/// Determine SDK Option value from flag pairs + clear flag.
/// - clear=true       → Some(vec![])      (remove all)
/// - non-empty pairs  → Some(entries)     (replace with these)
/// - empty + !clear   → None              (leave untouched on server)
pub(crate) fn resolve_policy_option(
    pairs: &[String],
    clear: bool,
) -> Result<Option<Vec<AccessPolicyEntry>>> {
    if clear && !pairs.is_empty() {
        bail!("Cannot use --clear-* and provide policies for the same category");
    }
    if clear {
        return Ok(Some(vec![]));
    }
    if pairs.is_empty() {
        return Ok(None);
    }
    Ok(Some(parse_policy_flags(pairs)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_permission_read() {
        assert_eq!(parse_permission("read").unwrap(), (true, false, false));
    }

    #[test]
    fn parse_permission_write() {
        assert_eq!(parse_permission("write").unwrap(), (true, true, false));
    }

    #[test]
    fn parse_permission_manage() {
        assert_eq!(parse_permission("manage").unwrap(), (true, true, true));
    }

    #[test]
    fn parse_permission_case_insensitive() {
        assert_eq!(parse_permission("MANAGE").unwrap(), (true, true, true));
        assert_eq!(parse_permission("Read").unwrap(), (true, false, false));
    }

    #[test]
    fn parse_permission_invalid() {
        assert!(parse_permission("admin").is_err());
        assert!(parse_permission("").is_err());
    }

    #[test]
    fn resolve_policy_option_clear() {
        let result = resolve_policy_option(&[], true).unwrap();
        let entries = result.expect("clear should return Some");
        assert!(entries.is_empty(), "clear should return empty vec");
    }

    #[test]
    fn resolve_policy_option_none() {
        let result = resolve_policy_option(&[], false).unwrap();
        assert!(result.is_none(), "no flags should return None");
    }

    #[test]
    fn resolve_policy_option_clear_with_entries_errors() {
        let uuid = uuid::Uuid::new_v4().to_string();
        let pairs = vec![uuid, "read".to_string()];
        assert!(resolve_policy_option(&pairs, true).is_err());
    }

    #[test]
    fn resolve_policy_option_entries() {
        let uuid = uuid::Uuid::new_v4();
        let pairs = vec![uuid.to_string(), "write".to_string()];
        let result = resolve_policy_option(&pairs, false).unwrap();
        let entries = result.expect("non-empty pairs should return Some");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].grantee_id, uuid);
        assert!(entries[0].read);
        assert!(entries[0].write);
        assert!(!entries[0].manage);
    }
}
