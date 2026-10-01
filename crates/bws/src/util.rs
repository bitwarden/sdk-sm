use regex::Regex;
use uuid::Uuid;

const VALID_POSIX_NAME_REGEX: &str = "^[a-zA-Z_][a-zA-Z0-9_]*$";
const STRING_TO_BOOL_ERROR_MESSAGE: &str = "Could not convert string to bool";

pub(crate) fn is_valid_posix_name(input_text: &str) -> bool {
    Regex::new(VALID_POSIX_NAME_REGEX)
        .expect("VALID_POSIX_NAME_REGEX to be a valid regex")
        .is_match(input_text)
}

pub(crate) fn string_to_bool(value: &str) -> Result<bool, &str> {
    match value.trim().to_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(STRING_TO_BOOL_ERROR_MESSAGE),
    }
}

/// Converts a UUID to a POSIX-compliant environment variable name.
///
/// POSIX environment variable names must start with a letter or an underscore
/// and can only contain letters, numbers, and underscores.
pub(crate) fn uuid_to_posix(uuid: &Uuid) -> String {
    format!("_{}", uuid.to_string().replace('-', "_"))
}

/// Whether `shell` refers to a PowerShell executable (`powershell` or `pwsh`).
fn is_powershell(shell: &str) -> bool {
    let file_name = shell.rsplit(['/', '\\']).next().unwrap_or(shell);
    let lowercased = file_name.to_lowercase();
    let stem = lowercased.strip_suffix(".exe").unwrap_or(&lowercased);
    stem.starts_with("powershell") || stem.starts_with("pwsh")
}

/// Joins command arguments into a single string that `shell -c` parses back
/// into the original arguments.
///
/// Each argument is single-quoted so the shell's parser treats it literally.
/// POSIX-compatible shells escape an embedded single quote as `'\''`, while
/// PowerShell doubles it and needs the `&` call operator to invoke a quoted
/// command name.
pub(crate) fn join_command_args(shell: &str, args: &[String]) -> String {
    if is_powershell(shell) {
        let quoted = args
            .iter()
            .map(|arg| format!("'{}'", arg.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(" ");
        format!("& {quoted}")
    } else {
        args.iter()
            .map(|arg| format!("'{}'", arg.replace('\'', "'\\''")))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_posix_name_true() {
        assert!(is_valid_posix_name("a_valid_name"));
        assert!(is_valid_posix_name("another_valid_name"));
        assert!(is_valid_posix_name("_another_valid_name"));
        assert!(is_valid_posix_name("ANOTHER_ONE"));
        assert!(is_valid_posix_name(
            "abcdefghijklmnopqrstuvwxyz__ABCDEFGHIJKLMNOPQRSTUVWXYZ__0123456789"
        ));
    }

    #[test]
    fn test_is_valid_posix_name_false() {
        assert!(!is_valid_posix_name(""));
        assert!(!is_valid_posix_name("1a"));
        assert!(!is_valid_posix_name("a bad name"));
        assert!(!is_valid_posix_name("another-bad-name"));
        assert!(!is_valid_posix_name("a\nbad\nname"));
    }

    #[test]
    fn test_uuid_to_posix_success() {
        assert_eq!(
            "_759130d0_29dd_48bd_831a_e3bdbafeeb6e",
            uuid_to_posix(
                &uuid::Uuid::parse_str("759130d0-29dd-48bd-831a-e3bdbafeeb6e").expect("valid uuid")
            )
        );
        assert!(is_valid_posix_name(&uuid_to_posix(&uuid::Uuid::new_v4())));
    }

    #[test]
    fn test_string_to_bool_true_true() {
        let result = string_to_bool("true");
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_string_to_bool_one_true() {
        let result = string_to_bool("1");
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_string_to_bool_false_false() {
        let result = string_to_bool("false");
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn test_string_to_bool_zero_false() {
        let result = string_to_bool("0");
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn test_string_to_bool_bad_string_errors() {
        let result = string_to_bool("hello world");
        assert_eq!(result, Err(STRING_TO_BOOL_ERROR_MESSAGE));
    }

    #[test]
    fn test_join_command_args_posix_preserves_boundaries() {
        let args = vec![
            "echo".to_string(),
            "hello world".to_string(),
            "three".to_string(),
        ];
        assert_eq!(
            "'echo' 'hello world' 'three'",
            join_command_args("sh", &args)
        );
    }

    #[test]
    fn test_join_command_args_posix_escapes_single_quotes() {
        let args = vec!["printf".to_string(), "don't".to_string()];
        assert_eq!("'printf' 'don'\\''t'", join_command_args("bash", &args));
    }

    #[test]
    fn test_join_command_args_posix_preserves_empty_arg() {
        let args = vec!["cmd".to_string(), String::new()];
        assert_eq!("'cmd' ''", join_command_args("/bin/sh", &args));
    }

    #[test]
    fn test_join_command_args_powershell_uses_call_operator() {
        let args = vec!["echo".to_string(), "a b".to_string()];
        assert_eq!("& 'echo' 'a b'", join_command_args("pwsh", &args));
    }

    #[test]
    fn test_join_command_args_powershell_escapes_single_quotes() {
        let args = vec!["echo".to_string(), "it's".to_string()];
        assert_eq!(
            "& 'echo' 'it''s'",
            join_command_args("powershell.exe", &args)
        );
    }

    #[test]
    fn test_is_powershell() {
        for shell in [
            "powershell",
            "powershell.exe",
            "pwsh",
            "/usr/bin/pwsh",
            r"C:\Program Files\PowerShell\7\pwsh.exe",
        ] {
            assert!(is_powershell(shell), "{shell} should be PowerShell");
        }
        for shell in ["sh", "/bin/bash", "zsh", "cmd", "cmd.exe"] {
            assert!(!is_powershell(shell), "{shell} should not be PowerShell");
        }
    }
}
