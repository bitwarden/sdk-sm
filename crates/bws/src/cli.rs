use std::path::PathBuf;

use bitwarden_cli::Color;
use clap::{ArgGroup, Parser, Subcommand, ValueEnum, builder::ValueParser};
use clap_complete::Shell;
use uuid::Uuid;

pub(crate) const ACCESS_TOKEN_KEY_VAR_NAME: &str = "BWS_ACCESS_TOKEN";
pub(crate) const CONFIG_FILE_KEY_VAR_NAME: &str = "BWS_CONFIG_FILE";
pub(crate) const PROFILE_KEY_VAR_NAME: &str = "BWS_PROFILE";
pub(crate) const SERVER_URL_KEY_VAR_NAME: &str = "BWS_SERVER_URL";
pub(crate) const UUIDS_AS_KEYNAMES_VAR_NAME: &str = "BWS_UUIDS_AS_KEYNAMES";

pub(crate) const DEFAULT_CONFIG_FILENAME: &str = "config";
pub(crate) const DEFAULT_CONFIG_DIRECTORY: &str = ".config/bws";

#[expect(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum ProfileKey {
    server_base,
    server_api,
    server_identity,
    state_dir,
    state_opt_out,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
#[expect(clippy::upper_case_acronyms)]
pub(crate) enum Output {
    JSON,
    YAML,
    Env,
    Table,
    TSV,
    None,
}

#[derive(Parser, Debug)]
#[command(name = "bws", version, about = "Bitwarden Secrets CLI", long_about = None)]
pub(crate) struct Cli {
    // Optional as a workaround for https://github.com/clap-rs/clap/issues/3572
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,

    #[arg(short = 'o', long, global = true, value_enum, default_value_t = Output::JSON, help="Output format")]
    pub(crate) output: Output,

    #[arg(short = 'c', long, global = true, value_enum, default_value_t = Color::Auto, help="Use colors in the output")]
    pub(crate) color: Color,

    #[arg(short = 't', long, global = true, env = ACCESS_TOKEN_KEY_VAR_NAME, hide_env_values = true, help="Specify access token for the service account")]
    pub(crate) access_token: Option<String>,

    #[arg(
        short = 'f',
        long,
        global = true,
        env = CONFIG_FILE_KEY_VAR_NAME,
        help = format!("[default: ~/{}/{}] Config file to use", DEFAULT_CONFIG_DIRECTORY, DEFAULT_CONFIG_FILENAME)
    )]
    pub(crate) config_file: Option<PathBuf>,

    #[arg(short = 'p', long, global = true, env = PROFILE_KEY_VAR_NAME, help="Profile to use from the config file")]
    pub(crate) profile: Option<String>,

    #[arg(short = 'u', long, global = true, env = SERVER_URL_KEY_VAR_NAME, help="Override the server URL from the config file", value_parser = ValueParser::new(url_parser) )]
    pub(crate) server_url: Option<String>,
}

fn url_parser(value: &str) -> Result<String, String> {
    if value.starts_with("http://") || value.starts_with("https://") {
        Ok(value.trim_end_matches('/').into())
    } else {
        Err(format!("'{value}' is not a valid URL"))
    }
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    #[command(long_about = "Configure the CLI", arg_required_else_help(true))]
    Config {
        name: Option<ProfileKey>,
        value: Option<String>,

        #[arg(short = 'd', long)]
        delete: bool,
    },

    #[command(long_about = "Generate shell completion files")]
    Completions { shell: Option<Shell> },

    #[command(long_about = "Commands available on Projects")]
    Project {
        #[command(subcommand)]
        cmd: ProjectCommand,
    },
    #[command(long_about = "Commands available on Secrets")]
    Secret {
        #[command(subcommand)]
        cmd: SecretCommand,
    },
    #[command(long_about = "Run a command with secrets injected")]
    Run {
        #[arg(help = "The command to run")]
        command: Vec<String>,
        #[arg(long, help = "The shell to use")]
        shell: Option<String>,
        #[arg(
            long,
            help = "Don't inherit environment variables from the current shell"
        )]
        no_inherit_env: bool,
        #[arg(long, help = "The ID of the project to use")]
        project_id: Option<Uuid>,
        #[arg(
            long,
            global = true,
            env = UUIDS_AS_KEYNAMES_VAR_NAME,
            help = "Use the secret UUID (in its POSIX form) instead of the key name for the environment variable"
        )]
        uuids_as_keynames: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum SecretCommand {
    Create {
        key: String,
        value: String,

        #[arg(help = "The ID of the project this secret will be added to")]
        project_id: Uuid,

        #[arg(long, help = "An optional note to add to the secret")]
        note: Option<String>,
    },
    Delete {
        secret_ids: Vec<Uuid>,
    },
    #[clap(group = ArgGroup::new("edit_field").required(true).multiple(true))]
    Edit {
        secret_id: Uuid,
        #[arg(long, group = "edit_field")]
        key: Option<String>,
        #[arg(long, group = "edit_field")]
        value: Option<String>,
        #[arg(long, group = "edit_field")]
        note: Option<String>,
        #[arg(long, group = "edit_field")]
        project_id: Option<Uuid>,
    },
    Get {
        secret_id: Uuid,
    },
    List {
        project_id: Option<Uuid>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum ProjectCommand {
    Create {
        name: String,
    },
    Delete {
        project_ids: Vec<Uuid>,
    },
    Edit {
        project_id: Uuid,
        #[arg(long, group = "edit_field")]
        name: String,
    },
    Get {
        project_id: Uuid,
    },
    List,
}
