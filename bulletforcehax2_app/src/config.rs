#![allow(warnings)]

use std::path::PathBuf;

use clap::{arg, command, value_parser, Arg, ArgMatches, Command, Parser};

const ARG_PROFILE_DIR: &str = "browser_profile";
const ARG_GAME_DIR: &str = "game_files";
const ARG_LOG_DIR: &str = "logs";

const DEFAULT_PROFILE_DIR: &str = "bfhax_data/browser_profile";
const DEFAULT_GAME_DIR: &str = "bfhax_data/game_files";
const DEFAULT_LOG_DIR: &str = "bfhax_data/logs";

// TODO: implement fallback to config
#[derive(Debug)]
pub struct Config {
    pub profile_dir: PathBuf,
    pub game_dir: PathBuf,
    pub log_dir: PathBuf,
}

pub struct PartialConfig {
    pub profile_dir: Option<PathBuf>,
    pub game_dir: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
}

impl Config {
    fn overwrite_with(self, new: PartialConfig) -> Config {
        Self {
            profile_dir: new.profile_dir.unwrap_or(self.profile_dir),
            game_dir: new.game_dir.unwrap_or(self.game_dir),
            log_dir: new.log_dir.unwrap_or(self.log_dir),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile_dir: PathBuf::from(DEFAULT_PROFILE_DIR),
            game_dir: PathBuf::from(DEFAULT_GAME_DIR),
            log_dir: PathBuf::from(DEFAULT_LOG_DIR),
        }
    }
}

impl From<ArgMatches> for PartialConfig {
    fn from(matches: ArgMatches) -> Self {
        Self {
            profile_dir: matches.get_one::<PathBuf>(ARG_PROFILE_DIR).cloned(),
            game_dir: matches.get_one::<PathBuf>(ARG_GAME_DIR).cloned(),
            log_dir: matches.get_one::<PathBuf>(ARG_LOG_DIR).cloned(),
        }
    }
}

/// Reads the config from the CLI arguments
pub fn get_config() -> Config {
    let config = Config::default();

    let matches = build_command().get_matches();
    let config = config.overwrite_with(PartialConfig::from(matches));

    config
}

fn build_command() -> Command {
    command!()
        .display_name("BulletForceHax2")
        .arg(
            Arg::new(ARG_PROFILE_DIR)
                .long("profile_dir")
                .value_name("PATH")
                .help(format!("Sets the path where the browser profile for the webview gets stored. [default: {DEFAULT_PROFILE_DIR}]"))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new(ARG_GAME_DIR)
                .long("game_dir")
                .value_name("PATH")
                .help(format!("Sets the path where downloaded game binaries get stored. [default: {DEFAULT_GAME_DIR}]"))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
            )
        .arg(
            Arg::new(ARG_LOG_DIR)
                .long("log_dir")
                .value_name("PATH")
                .help(format!("Sets the path where log files get stored. [default: {DEFAULT_LOG_DIR}]"))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
}
