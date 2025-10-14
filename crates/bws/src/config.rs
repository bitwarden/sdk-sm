use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Result, bail};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use crate::cli::{DEFAULT_CONFIG_DIRECTORY, DEFAULT_CONFIG_FILENAME, ProfileKey};

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct Config {
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
    pub state_opt_out: Option<String>,
}

fn deserialize_trimmed_url<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt_string: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt_string.map(|s| s.trim_end_matches('/').to_string()))
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
            ProfileKey::state_opt_out => p.state_opt_out = Some(value),
        }
    }
}

fn get_config_path(config_file: Option<&Path>, ensure_folder_exists: bool) -> Result<PathBuf> {
    let config_file = match config_file {
        Some(path) => path.to_owned(),
        None => {
            let Some(base_dirs) = BaseDirs::new() else {
                bail!("A valid home directory doesn't exist");
            };
            base_dirs
                .home_dir()
                .join(DEFAULT_CONFIG_DIRECTORY)
                .join(DEFAULT_CONFIG_FILENAME)
        }
    };

    if ensure_folder_exists {
        if let Some(parent_folder) = config_file.parent() {
            std::fs::create_dir_all(parent_folder)?;
        }
    }

    Ok(config_file)
}

pub(crate) fn load_config(config_file: Option<&Path>, must_exist: bool) -> Result<Config> {
    let file = get_config_path(config_file, false)?;

    let content = match file.exists() {
        true => read_to_string(file),
        false if must_exist => bail!("Config file doesn't exist"),
        false => return Ok(Config::default()),
    };

    let config: Config = toml::from_str(&content?)?;
    Ok(config)
}

fn write_config(config: Config, config_file: Option<&Path>) -> Result<()> {
    let file = get_config_path(config_file, true)?;

    let content = toml::to_string_pretty(&config)?;

    std::fs::write(file, content)?;
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
    let mut config = load_config(config_file, true)?;

    if !config.profiles.contains_key(&profile) {
        bail!("Profile does not exist");
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

    pub(crate) fn api_url(&self) -> Result<String> {
        if let Some(api) = &self.server_api {
            return Ok(api.clone());
        }

        if let Some(base) = &self.server_base {
            return Ok(format!("{base}/api"));
        }

        bail!("Profile has no `server_base` or `server_api`");
    }

    pub(crate) fn identity_url(&self) -> Result<String> {
        if let Some(identity) = &self.server_identity {
            return Ok(identity.clone());
        }

        if let Some(base) = &self.server_base {
            return Ok(format!("{base}/identity"));
        }

        bail!("Profile has no `server_base` or `server_identity`");
    }
}

impl Config {
    pub(crate) fn select_profile(
        &self,
        profile: &str,
        profile_defined: bool,
    ) -> Result<Option<Profile>> {
        if let Some(profile) = self.profiles.get(profile) {
            return Ok(Some(profile.clone()));
        }

        if profile_defined {
            bail!("The specified profile does not exist");
        }

        if let Some(profile) = self.profiles.get("default") {
            return Ok(Some(profile.clone()));
        }

        Ok(None)
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
        assert!(c.is_err());

        let c = load_config(None, false);
        assert!(c.is_ok());
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
