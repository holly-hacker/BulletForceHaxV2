use std::path::PathBuf;

use clap::{command, parser::ValueSource, value_parser, Arg, ArgAction, ArgMatches, Command};
use serde::{Deserialize, Serialize};

// NOTE: these values are copied to the `serde(rename)` attributes in PartialConfig
const ARG_CONFIG_FILE: Opt<&str> = opt("config", "config.toml");
const ARG_PORT: Opt<u16> = opt("port", 48897);
const ARG_PROFILE_DIR: Opt<&str> = opt("browser-profile", "bfhax_data/browser_profile");
const ARG_GAME_DIR: Opt<&str> = opt("game-files", "bfhax_data/game_files");
const ARG_LOG_DIR: Opt<&str> = opt("logs", "bfhax_data/logs");
const ARG_OPEN_DEVTOOLS: Opt<bool> = opt("open-devtools", false);
const ARG_HAX: Opt<bool> = opt("hax", false);

#[derive(Debug, Clone, Serialize)]
pub struct Config {
    pub config_file: PathBuf,
    pub port: u16,
    pub profile_dir: PathBuf,
    pub game_dir: PathBuf,
    pub log_dir: PathBuf,
    pub open_devtools: bool,
    pub hax: bool,
}

struct Opt<T> {
    name: &'static str,
    value: T,
}

const fn opt<T>(name: &'static str, value: T) -> Opt<T> {
    Opt { name, value }
}

#[derive(Default, Deserialize)]
pub struct PartialConfig {
    #[serde(skip)]
    pub config_file: Option<PathBuf>,
    #[serde(rename = "port")]
    pub port: Option<u16>,
    #[serde(rename = "browser-profile")]
    pub profile_dir: Option<PathBuf>,
    #[serde(rename = "game-files")]
    pub game_dir: Option<PathBuf>,
    #[serde(rename = "logs")]
    pub log_dir: Option<PathBuf>,
    #[serde(rename = "open-devtools")]
    pub open_devtools: Option<bool>,
    #[serde(rename = "hax")]
    pub hax: Option<bool>,
}

impl Config {
    fn overwrite_with(self, new: PartialConfig) -> Config {
        Self {
            config_file: new.config_file.unwrap_or(self.config_file),
            port: new.port.unwrap_or(self.port),
            profile_dir: new.profile_dir.unwrap_or(self.profile_dir),
            game_dir: new.game_dir.unwrap_or(self.game_dir),
            log_dir: new.log_dir.unwrap_or(self.log_dir),
            open_devtools: new.open_devtools.unwrap_or(self.open_devtools),
            hax: new.hax.unwrap_or(self.hax),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_file: PathBuf::from(ARG_CONFIG_FILE.value),
            port: ARG_PORT.value,
            profile_dir: PathBuf::from(ARG_PROFILE_DIR.value),
            game_dir: PathBuf::from(ARG_GAME_DIR.value),
            log_dir: PathBuf::from(ARG_LOG_DIR.value),
            open_devtools: ARG_OPEN_DEVTOOLS.value,
            hax: ARG_HAX.value,
        }
    }
}

impl From<ArgMatches> for PartialConfig {
    fn from(matches: ArgMatches) -> Self {
        Self {
            config_file: matches.get_one::<PathBuf>(ARG_CONFIG_FILE.name).cloned(),
            port: matches.get_one::<u16>(ARG_PORT.name).cloned(),
            profile_dir: matches.get_one::<PathBuf>(ARG_PROFILE_DIR.name).cloned(),
            game_dir: matches.get_one::<PathBuf>(ARG_GAME_DIR.name).cloned(),
            log_dir: matches.get_one::<PathBuf>(ARG_LOG_DIR.name).cloned(),
            open_devtools: (matches.value_source(ARG_OPEN_DEVTOOLS.name)
                == Some(ValueSource::CommandLine))
            .then(|| {
                matches
                    .get_one::<bool>(ARG_OPEN_DEVTOOLS.name)
                    .cloned()
                    .unwrap()
            }),
            hax: (matches.value_source(ARG_HAX.name) == Some(ValueSource::CommandLine))
                .then(|| matches.get_one::<bool>(ARG_HAX.name).cloned().unwrap()),
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
            Arg::new(ARG_CONFIG_FILE.name)
                .long(ARG_CONFIG_FILE.name)
                .value_name("PATH")
                .help(format!("Specifies which config file should be read. [default: {}]", ARG_CONFIG_FILE.value))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new(ARG_PORT.name)
                .long(ARG_PORT.name)
                .value_name("PORT")
                .help(format!("Specifies the port to run the web server on. [default: {}]", ARG_PORT.value))
                .required(false)
                .value_parser(value_parser!(u16)),
        )
        .arg(
            Arg::new(ARG_HAX.name)
                .long(ARG_HAX.name)
                .help("Enable cheat functionality.")
                .required(false)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(ARG_PROFILE_DIR.name)
                .long(ARG_PROFILE_DIR.name)
                .value_name("PATH")
                .help(format!("Sets the path where the browser profile for the webview gets stored. [default: {}]", ARG_PROFILE_DIR.value))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new(ARG_GAME_DIR.name)
                .long(ARG_GAME_DIR.name)
                .value_name("PATH")
                .help(format!("Sets the path where downloaded game binaries get stored. [default: {}]", ARG_GAME_DIR.value))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
            )
        .arg(
            Arg::new(ARG_LOG_DIR.name)
                .long(ARG_LOG_DIR.name)
                .value_name("PATH")
                .help(format!("Sets the path where log files get stored. [default: {}]", ARG_LOG_DIR.value))
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new(ARG_OPEN_DEVTOOLS.name)
                .long(ARG_OPEN_DEVTOOLS.name)
                .help("Automatically open the webview's devtools on start.")
                .required(false)
                .action(ArgAction::SetTrue),
        )
}
