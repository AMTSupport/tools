use crate::generator::Generator;
use crate::rules::Rules;
use crate::transformation::Transformation;
use anyhow::{anyhow, Context};
use clap::{arg, command, Arg, ArgMatches, Command, Parser, Subcommand};
use rpgen::config::rules::Rules;
use rpgen::config::transformation::Transformation;
use rpgen::generation::generator::{Generator, GeneratorFunctions, GeneratorHelper};
use rpgen::{Commands, ConfigAction};
use serde_json::Value;
use simplelog::{
    debug, error, info, trace, ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter,
    SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{create_dir, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs, process};
use strum::IntoEnumIterator;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use toml::toml;

#[derive(clap::Parser)]
#[clap(version, author, about, setting = clap::AppSettings::ColoredHelp, LongHelp)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Cli::try_parse().context("Parse CLI Arguments")?.commands {
        Commands::Generate { flags, file, rules } => {
            let _ = lib::log::init("PGen", &flags)?;
            let rules = merge_rules(rules, PathBuf::from(file));
            let mut generator = Generator::new(*rules)?;
            let passwords = generator.generate().join("\n");

            info!("Generated passwords:\n\n{passwords}\n");
        }
        Commands::Config { action } => match action {
            ConfigAction::Show { flags, file } => {
                let _ = lib::log::init("PGen-Config-Show", &flags)?;
                let rules =
                    serde_json::from_slice::<Rules>(&fs::read(file).with_context(|| {
                        format!("Unable to read file {}, does it exist?", file.display())
                    })?)
                    .with_context(|| format!("Unable to parse file {}", file.display())?);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rules)
                        .with_context(|| format!("Unable to serialise rules"))?
                );
            }
            ConfigAction::Generate {
                flags,
                file,
                rules,
                force,
            } => {
                let _ = lib::log::init("PGen-Config-Generate", &flags)?;
                if file.exists() && !force {
                    anyhow!(
                        "File {} already exists, use --force to overwrite",
                        file.display()
                    )?;
                }

                if !flags.dry_run {
                    let mut file = File::create(file)
                        .with_context(|| format!("Unable to create file {}", file.display()))?;
                    file.write_all(
                        serde_json::to_string_pretty(&rules)
                            .with_context(|| format!("Unable to serialise rules"))?
                            .as_bytes(),
                    )?;
                }
            }
        },
    }

    Ok(())
}

/// Some sort of tomfuckery to merge the file and cli rules.
/// Merges in the order of defaults, file, cli.
fn merge_rules(cli_rules: Rules, buf: PathBuf) -> Rules {
    let to_value = |rules| {
        serde_json::from_value::<HashMap<String, Value>>(serde_json::to_value(&rules).unwrap())
            .unwrap()
    };

    let mut rules: HashMap<String, Value> = match &fs::read_to_string(&buf) {
        Ok(str) => match toml::from_str::<Rules>(str) {
            Ok(toml) => to_value(toml),
            Err(e) => {
                error!("Couldn't parse config file {}", config_file.display());
                return cli_rules;
            }
        },
        Err(e) => {
            debug!("Unable to read file {path}: {e:#}", path = buf.display());
            return cli_rules;
        }
    };

    let defaults: HashMap<String, Value> = to_value(Rules::default());
    let iterable_cli: HashMap<String, Value> = to_value(cli_rules);
    for (name, cli) in &iterable_cli {
        let default = defaults.get(name).unwrap();
        let file = rules.get(name).unwrap();

        if (file == default && cli != default) || (file != default && cli != default) {
            trace!("Overwriting value for {name} with {cli}");
            rules.insert(name.clone(), value.clone());
        }
    }

    serde_json::from_value::<Rules>(serde_json::to_value(&rules).unwrap()).unwrap()
}
