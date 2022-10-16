use std::path::PathBuf;

use clap::{command, value_parser, Arg, ArgAction, ArgMatches, Command};
use serde::Deserialize;

// NOTE: these values are copied to the `serde(rename)` attributes in PartialConfig
const ARG_CONFIG_FILE: &str = "config";
const ARG_PROFILE_DIR: &str = "browser-profile";
const ARG_GAME_DIR: &str = "game-files";
const ARG_LOG_DIR: &str = "logs";
const ARG_OPEN_DEVTOOLS: &str = "open-devtools";

const DEFAULT_CONFIG_FILE: &str = "config.toml";
const DEFAULT_PROFILE_DIR: &str = "bfhax_data/browser_profile";
const DEFAULT_GAME_DIR: &str = "bfhax_data/game_files";
const DEFAULT_LOG_DIR: &str = "bfhax_data/logs";
const DEFAULT_OPEN_DEVTOOLS: bool = false;

#[derive(Debug)]
pub struct Config {
    pub config_file: PathBuf,
    pub profile_dir: PathBuf,
    pub game_dir: PathBuf,
    pub log_dir: PathBuf,
    pub open_devtools: bool,
}

#[derive(Default, Deserialize)]
pub struct PartialConfig {
    #[serde(skip)]
    pub config_file: Option<PathBuf>,
    #[serde(rename = "browser-profile")]
    pub profile_dir: Option<PathBuf>,
    #[serde(rename = "game-files")]
    pub game_dir: Option<PathBuf>,
    #[serde(rename = "logs")]
    pub log_dir: Option<PathBuf>,
    #[serde(rename = "open-devtools")]
    pub open_devtools: Option<bool>,
}

impl Config {
    fn overwrite_with(self, new: PartialConfig) -> Config {
        Self {
            config_file: new.config_file.unwrap_or(self.config_file),
            profile_dir: new.profile_dir.unwrap_or(self.profile_dir),
            game_dir: new.game_dir.unwrap_or(self.game_dir),
            log_dir: new.log_dir.unwrap_or(self.log_dir),
            open_devtools: new.open_devtools.unwrap_or(self.open_devtools),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_file: PathBuf::from(DEFAULT_CONFIG_FILE),
            profile_dir: PathBuf::from(DEFAULT_PROFILE_DIR),
            game_dir: PathBuf::from(DEFAULT_GAME_DIR),
            log_dir: PathBuf::from(DEFAULT_LOG_DIR),
            open_devtools: DEFAULT_OPEN_DEVTOOLS,
        }
    }
}

impl From<ArgMatches> for PartialConfig {
    fn from(matches: ArgMatches) -> Self {
        Self {
            config_file: matches.get_one::<PathBuf>(ARG_CONFIG_FILE).cloned(),
            profile_dir: matches.get_one::<PathBuf>(ARG_PROFILE_DIR).cloned(),
            game_dir: matches.get_one::<PathBuf>(ARG_GAME_DIR).cloned(),
            log_dir: matches.get_one::<PathBuf>(ARG_LOG_DIR).cloned(),
            open_devtools: matches.get_one::<bool>(ARG_OPEN_DEVTOOLS).cloned(),
        }
    }
}

/// Reads the config from the CLI arguments
pub fn get_config() -> Config {
    let config = Config::default();

    let matches = build_command().get_matches();
    let cli_config = PartialConfig::from(matches);

    let config_path = cli_config
        .config_file
        .as_ref()
        .unwrap_or(&config.config_file);
    let config_file = std::fs::read(config_path).ok();
    let file_config = config_file
        .and_then(|v| toml::from_slice(&v).ok())
        .unwrap_or_default();

    config
        .overwrite_with(file_config)
        .overwrite_with(cli_config)
}

fn build_command() -> Command {
    command!()
        .display_name("BulletForceHax2")
        .arg(
            Arg::new(ARG_CONFIG_FILE)
                .long(ARG_CONFIG_FILE)
                .value_name("PATH")
                .help(format!("Specifies which config file should be read. [default: {DEFAULT_CONFIG_FILE}]"))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new(ARG_PROFILE_DIR)
                .long(ARG_PROFILE_DIR)
                .value_name("PATH")
                .help(format!("Sets the path where the browser profile for the webview gets stored. [default: {DEFAULT_PROFILE_DIR}]"))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new(ARG_GAME_DIR)
                .long(ARG_GAME_DIR)
                .value_name("PATH")
                .help(format!("Sets the path where downloaded game binaries get stored. [default: {DEFAULT_GAME_DIR}]"))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
            )
        .arg(
            Arg::new(ARG_LOG_DIR)
                .long(ARG_LOG_DIR)
                .value_name("PATH")
                .help(format!("Sets the path where log files get stored. [default: {DEFAULT_LOG_DIR}]"))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new(ARG_OPEN_DEVTOOLS)
                .long(ARG_OPEN_DEVTOOLS)
                .help(format!("Automatically open the webview's devtools on start."))
                .required(false)
                .action(ArgAction::SetTrue),
        )
}
