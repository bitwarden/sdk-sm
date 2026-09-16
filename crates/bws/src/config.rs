use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Result, bail};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{DEFAULT_CONFIG_DIRECTORY, DEFAULT_CONFIG_FILENAME, ProfileKey},
    error::{self, UserError},
    util::string_to_bool,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct Config {
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct Profile {
    #[serde(deserialize_with = "deserialize_trimmed_url", default)]
    pub server_base: Option<String>,
    #[serde(deserialize_with = "deserialize_trimmed_url", default)]
    pub server_api: Option<String>,
    #[serde(deserialize_with = "deserialize_trimmed_url", default)]
    pub server_identity: Option<String>,
    pub state_dir: Option<String>,
    #[serde(deserialize_with = "allow_string", default)]
    pub state_opt_out: Option<bool>,
}

fn deserialize_trimmed_url<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt_string: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt_string.map(|s| s.trim_end_matches('/').to_string()))
}

fn allow_string<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: Option<toml::Value> = Option::deserialize(deserializer)?;
    val.map(|v| -> Result<bool, D::Error> {
        match v {
            toml::Value::Boolean(s) => Ok(s),
            toml::Value::Integer(n) => Ok(n != 0),
            toml::Value::String(s) => Ok(string_to_bool(&s).unwrap_or_default()),
            _ => Err(<D::Error as serde::de::Error>::custom(
                "only bools and strings are accepted",
            )),
        }
    })
    .transpose()
}

impl ProfileKey {
    fn update_profile_value(&self, p: &mut Profile, value: String) {
        let value = if matches!(
            self,
            ProfileKey::server_base | ProfileKey::server_api | ProfileKey::server_identity
        ) {
            value.trim_end_matches('/').to_string()
        } else {
            value
        };

        match self {
            ProfileKey::server_base => p.server_base = Some(value),
            ProfileKey::server_api => p.server_api = Some(value),
            ProfileKey::server_identity => p.server_identity = Some(value),
            ProfileKey::state_dir => p.state_dir = Some(value),
            ProfileKey::state_opt_out => {
                p.state_opt_out = Some(string_to_bool(&value).unwrap_or_default())
            }
        }
    }
}

pub(crate) fn get_config_path(
    config_file: Option<&Path>,
    ensure_folder_exists: bool,
) -> Result<PathBuf> {
    let config_file = match config_file {
        Some(path) => path.to_owned(),
        None => {
            let Some(base_dirs) = BaseDirs::new() else {
                return Err(UserError::new("Could not determine the home directory.")
                    .hint("Set --config-file or BWS_CONFIG_FILE.")
                    .into());
            };
            base_dirs
                .home_dir()
                .join(DEFAULT_CONFIG_DIRECTORY)
                .join(DEFAULT_CONFIG_FILENAME)
        }
    };

    if ensure_folder_exists && let Some(parent_folder) = config_file.parent() {
        std::fs::create_dir_all(parent_folder)
            .map_err(|e| UserError::io("Could not create config directory", parent_folder, e))?;
    }

    Ok(config_file)
}

pub(crate) fn load_config(config_file: Option<&Path>, must_exist: bool) -> Result<Config> {
    load_config_at(&get_config_path(config_file, false)?, must_exist)
}

/// Like [`load_config`], but for an already resolved config file path.
pub(crate) fn load_config_at(file: &Path, must_exist: bool) -> Result<Config> {
    if file.is_dir() {
        return Err(UserError::new(format!(
            "Config file '{}' is a directory.",
            file.display()
        ))
        .hint("In a container, create the host file first (e.g. `touch`) so it mounts as a file.")
        .into());
    }

    if !file.exists() {
        if must_exist {
            return Err(UserError::new(format!(
                "Config file '{}' does not exist.",
                file.display()
            ))
            .into());
        }
        return Ok(Config::default());
    }

    let content =
        read_to_string(file).map_err(|e| UserError::io("Could not read config file", file, e))?;

    let config: Config = toml::from_str(&content).map_err(|e| toml_error(file, &content, e))?;
    Ok(config)
}

fn toml_error(file: &Path, content: &str, e: toml::de::Error) -> UserError {
    let reason = error::clause(&e.message().replace("struct Profile", "a profile table"));
    let message = match e.span() {
        Some(span) => {
            let before = content.get(..span.start).unwrap_or(content);
            let line = before.matches('\n').count() + 1;
            let column = before
                .rsplit_once('\n')
                .map_or(before, |(_, last)| last)
                .chars()
                .count()
                + 1;
            format!(
                "Invalid config file '{}' at line {line}, column {column}: {reason}.",
                file.display()
            )
        }
        None => format!("Invalid config file '{}': {reason}.", file.display()),
    };
    UserError::new(message).source(e)
}

fn write_config(config: Config, config_file: Option<&Path>) -> Result<()> {
    let file = get_config_path(config_file, true)?;

    let content = toml::to_string_pretty(&config)?;

    std::fs::write(&file, content)
        .map_err(|e| UserError::io("Could not write config file", &file, e))?;
    Ok(())
}

pub(crate) fn update_profile(
    config_file: Option<&Path>,
    profile: String,
    name: ProfileKey,
    value: String,
) -> Result<()> {
    let mut config = load_config(config_file, false)?;

    let p = config.profiles.entry(profile).or_default();

    if value.starts_with("http://") || value.starts_with("https://") {
        name.update_profile_value(p, value.trim_end_matches('/').to_string());
    } else {
        name.update_profile_value(p, value);
    }

    write_config(config, config_file)?;
    Ok(())
}

pub(crate) fn delete_profile(config_file: Option<&Path>, profile: String) -> Result<()> {
    let path = get_config_path(config_file, false)?;
    if !path.exists() {
        return Err(UserError::new(format!(
            "Config file '{}' does not exist; nothing to delete.",
            path.display()
        ))
        .into());
    }
    let mut config = load_config(Some(&path), true)?;

    if !config.profiles.contains_key(&profile) {
        return Err(UserError::new(format!(
            "Profile '{profile}' not found in '{}'.",
            path.display()
        ))
        .into());
    }

    config.profiles.remove(&profile);

    write_config(config, config_file)?;
    Ok(())
}

impl Profile {
    pub(crate) fn from_url(url: &str) -> Result<Profile> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            bail!("Server URL must start with http:// or https://, the provided URL is: `{url}`");
        }

        Ok(Profile {
            server_base: Some(url.to_string()),
            server_api: None,
            server_identity: None,
            state_dir: None,
            state_opt_out: None,
        })
    }

    /// Returns the identity and API URLs of the profile named `name`.
    pub(crate) fn server_urls(&self, name: &str) -> Result<(String, String), UserError> {
        // An explicit URL wins over `<server_base>/<path>`.
        let url = |explicit: &Option<String>, path: &str| {
            explicit.clone().or_else(|| {
                self.server_base
                    .as_ref()
                    .map(|base| format!("{base}/{path}"))
            })
        };

        url(&self.server_identity, "identity")
            .zip(url(&self.server_api, "api"))
            .ok_or_else(|| {
                UserError::new(format!("Profile '{name}' has no server URL.")).hint(format!(
                    "Run: bws config --profile {name} server-base <url>"
                ))
            })
    }
}

impl Config {
    /// Returns the name and contents of the profile to use, falling back to `default` unless
    /// the profile was explicitly requested.
    pub(crate) fn select_profile(
        &self,
        profile: &str,
        profile_defined: bool,
        config_file: &Path,
    ) -> Result<Option<(String, Profile)>, UserError> {
        if let Some(p) = self.profiles.get(profile) {
            return Ok(Some((profile.to_string(), p.clone())));
        }

        if profile_defined {
            return Err(UserError::new(format!(
                "Profile '{profile}' not found in '{}'.",
                config_file.display()
            )));
        }

        Ok(self
            .profiles
            .get("default")
            .map(|p| ("default".to_string(), p.clone())))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn config_doesnt_exist() {
        let c = load_config(Some(Path::new("non_existing")), true);
        assert_eq!(
            c.expect_err("missing config file").to_string(),
            "Config file 'non_existing' does not exist."
        );

        let c = load_config(None, false);
        assert!(c.is_ok());
    }

    #[test]
    fn config_empty_file_is_valid() {
        let tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile.as_file(), "").unwrap();

        let c = load_config(Some(Path::new(tmpfile.as_ref())), true);
        let config = c.unwrap();
        assert_eq!(0, config.profiles.len());
    }

    #[test]
    fn config_state_opt_out_as_boolean() {
        let tmpfile = NamedTempFile::new().unwrap();
        write!(
            tmpfile.as_file(),
            "[profiles.default]\nstate_opt_out = true\n"
        )
        .unwrap();

        let c = load_config(Some(Path::new(tmpfile.as_ref())), true);
        assert_eq!(Some(true), c.unwrap().profiles["default"].state_opt_out);
    }

    #[test]
    fn config_state_opt_out_as_string() {
        let tmpfile = NamedTempFile::new().unwrap();
        write!(
            tmpfile.as_file(),
            "[profiles.default]\nstate_opt_out = \"false\"\n"
        )
        .unwrap();

        let c = load_config(Some(Path::new(tmpfile.as_ref())), true);
        assert_eq!(Some(false), c.unwrap().profiles["default"].state_opt_out);
    }

    #[test]
    fn config_exist() {
        let tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile.as_file(), "[profiles]").unwrap();

        let c = load_config(Some(Path::new(tmpfile.as_ref())), true);
        let config = c.unwrap();
        assert_eq!(0, config.profiles.len());
    }

    #[test]
    fn config_exist_with_profile() {
        let tmpfile = NamedTempFile::new().unwrap();
        write!(
            tmpfile.as_file(),
            "[profiles.default]
        server_base = \"https://bitwarden.com\"
        "
        )
        .unwrap();

        let c = load_config(Some(Path::new(tmpfile.as_ref())), true);
        assert_eq!(
            "https://bitwarden.com",
            c.unwrap().profiles["default"].server_base.as_ref().unwrap()
        );
    }

    #[test]
    fn config_invalid_toml() {
        let tmpfile = NamedTempFile::new().expect("temp file to be created");
        write!(tmpfile.as_file(), "[profiles.default]\nhello").expect("temp file to be written");

        let e = load_config(Some(tmpfile.path()), true).expect_err("invalid TOML");
        assert_eq!(
            e.to_string(),
            format!(
                "Invalid config file '{}' at line 2, column 6: key with no value, expected `=`.",
                tmpfile.path().display()
            )
        );
    }

    #[test]
    fn config_profile_wrong_type() {
        let tmpfile = NamedTempFile::new().expect("temp file to be created");
        write!(tmpfile.as_file(), "[profiles]\ndefault = \"x\"").expect("temp file to be written");

        let e = load_config(Some(tmpfile.path()), true).expect_err("profile is not a table");
        assert_eq!(
            e.to_string(),
            format!(
                "Invalid config file '{}' at line 2, column 11: invalid type: string \"x\", expected a profile table.",
                tmpfile.path().display()
            )
        );
    }

    #[test]
    fn config_unknown_profile() {
        let config = Config::default();
        let e = config
            .select_profile("work", true, Path::new("/tmp/bws.toml"))
            .expect_err("unknown profile");
        assert_eq!(
            e.to_string(),
            "Profile 'work' not found in '/tmp/bws.toml'."
        );

        let profile = config
            .select_profile("work", false, Path::new("/tmp/bws.toml"))
            .expect("implicit profile to fall back");
        assert!(profile.is_none());
    }

    #[test]
    fn profile_without_urls() {
        let profile = Profile {
            server_identity: Some("https://identity.example.com".to_string()),
            ..Default::default()
        };
        let e = profile.server_urls("work").expect_err("no API URL");
        assert_eq!(e.to_string(), "Profile 'work' has no server URL.");
        assert_eq!(
            e.hint_text(),
            Some("Run: bws config --profile work server-base <url>")
        );

        let profile = Profile {
            server_base: Some("https://example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(
            profile.server_urls("work").expect("URLs from server_base"),
            (
                "https://example.com/identity".to_string(),
                "https://example.com/api".to_string()
            )
        );
    }

    #[test]
    fn config_trims_trailing_forward_slashes_in_urls() {
        let tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile.as_file(), "[profiles.default]").unwrap();

        let _ = update_profile(
            Some(tmpfile.as_ref()),
            "default".to_owned(),
            ProfileKey::server_base,
            "https://vault.bitwarden.com//////".to_owned(),
        );

        let _ = update_profile(
            Some(tmpfile.as_ref()),
            "default".to_owned(),
            ProfileKey::server_api,
            "https://api.bitwarden.com/".to_owned(),
        );

        let _ = update_profile(
            Some(tmpfile.as_ref()),
            "default".to_owned(),
            ProfileKey::server_identity,
            "https://identity.bitwarden.com/".to_owned(),
        );

        let c = load_config(Some(Path::new(tmpfile.as_ref())), true).unwrap();
        assert_eq!(
            "https://vault.bitwarden.com",
            c.profiles["default"].server_base.as_ref().unwrap()
        );
        assert_eq!(
            "https://api.bitwarden.com",
            c.profiles["default"].server_api.as_ref().unwrap()
        );
        assert_eq!(
            "https://identity.bitwarden.com",
            c.profiles["default"].server_identity.as_ref().unwrap()
        );
    }

    #[test]
    fn config_does_not_trim_forward_slashes_in_non_url_values() {
        let tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile.as_file(), "[profiles.default]").unwrap();

        let _ = update_profile(
            Some(tmpfile.as_ref()),
            "default".to_owned(),
            ProfileKey::state_dir,
            "/dev/null/".to_owned(),
        );

        let c = load_config(Some(Path::new(tmpfile.as_ref())), true).unwrap();
        assert_eq!(
            "/dev/null/",
            c.profiles["default"].state_dir.as_ref().unwrap()
        );
    }
}
