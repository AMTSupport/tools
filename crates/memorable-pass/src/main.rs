/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#![feature(exclusive_range_pattern)]
#![feature(async_fn_in_trait)]

use clap::{Parser, Subcommand};
use lib::cli::Flags;
use lib::ui::cli::cli::{AsyncCliUI, CliResult, CliUI};
use memorable_pass::processor::processor::Processor;
use memorable_pass::rules::rule::Rule;
use memorable_pass::rules::rules::Rules;
use memorable_pass::{config, random_words};
use tokio::runtime::Handle;
use tracing::{debug, info};

#[derive(Debug, Parser)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct CliOneShot {
    #[command(flatten)]
    pub flags: Flags,

    #[command(flatten)]
    pub rules: Rules,

    #[arg(long)]
    pub repl: bool,
}

#[derive(Debug, Parser)]
#[command(name = env!["CARGO_PKG_NAME"], version, author, about)]
pub struct CliRepl {
    #[command(flatten)]
    pub flags: Flags,

    #[command(subcommand)]
    pub action: ReplCommand,
}

#[derive(Debug, Subcommand)]
pub enum ReplCommand {
    Generate,
    Rules(Rules),

    Ping,
    #[command(alias = "q")]
    Quit,
}

pub struct App {
    rules: Option<Rules>,
}

impl CliUI for App {
    type OneShotCommand = CliOneShot;
    type ReplCommand = CliRepl;

    fn new(_args: Self::Args) -> CliResult<Self>
    where
        Self: Sized,
    {
        // Preload the words
        Handle::current().spawn(async {
            let _preload = &config::asset::WORDS;
        });

        Ok(Self { rules: None })
    }
}

impl AsyncCliUI for App {
    async fn handle_command(&mut self, command: Self::OneShotCommand) -> CliResult<()> {
        self.rules.replace(command.rules);
        let passwords = generate(self.rules.as_ref().unwrap()).await;
        info!(
            "Generated passwords:\n\n{passwords}\n",
            passwords = passwords.join("\n")
        );

        Ok(())
    }

    async fn handle_repl_command(&mut self, command: Self::ReplCommand) -> CliResult<bool> {
        match command.action {
            ReplCommand::Generate => {
                let rules = self.rules.get_or_insert_with(|| {
                    debug!("No rules set, using defaults");
                    Rules::default()
                });

                let passwords = generate(&rules).await;
                info!(
                    "Generated passwords:\n\n{passwords}\n",
                    passwords = passwords.join("\n")
                );
            }
            ReplCommand::Rules(rules) => {
                let previous_rules = self.rules.replace(rules);

                if let Some(previous_rules) = previous_rules {
                    debug!("Replacing previous rules.");
                    debug!("Previous rules:\n\n{previous_rules:?}\n");
                }
            }
            ReplCommand::Ping => {
                info!("Pong!");
            }
            ReplCommand::Quit => {
                info!("Quitting...");
                return Ok(true);
            }
        }

        Ok(false)
    }
}

async fn generate(rules: &Rules) -> Vec<String> {
    let mut passwords = Vec::with_capacity(rules.amount);
    while passwords.len() < rules.amount {
        let words = random_words(rules.word_count, rules.word_length_min, rules.word_length_max).await;
        let mut processor = Processor::new(words);
        rules.addition_digits.process(&mut processor);
        rules.addition_separator.process(&mut processor);
        rules.transformation_case.process(&mut processor);

        passwords.push(processor.finish());
    }

    passwords
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::new(())?;
    let (repl, verbosity) = CliOneShot::try_parse().map(|cli| (cli.repl, cli.flags.verbose))?;
    let _guard = lib::log::init(env!("CARGO_PKG_NAME"), verbosity);

    match repl {
        true => app.repl().await,
        false => app.run().await,
    }?;

    // let cli = Cli::parse();
    // let _guard = lib::log::init(env!("CARGO_PKG_NAME"), cli.flags.verbose);
    //
    // debug!("CLI: {:#?}", cli);
    //
    // let mut passwords = Vec::with_capacity(cli.rules.amount);
    // while passwords.len() < cli.rules.amount {
    //     let words = random_words(
    //         cli.rules.word_count,
    //         cli.rules.word_length_min,
    //         cli.rules.word_length_max,
    //     )
    //     .await;
    //     let mut processor = Processor::new(words);
    //     cli.rules.addition_digits.process(&mut processor);
    //     cli.rules.addition_separator.process(&mut processor);
    //     cli.rules.transformation_case.process(&mut processor);
    //
    //     passwords.push(processor.finish());
    // }
    //
    // info!(
    //     "Generated passwords:\n\n{passwords}\n",
    //     passwords = passwords.join("\n")
    // );

    // match cli.command {
    //     Commands::Generate { flags, file, rules } => {
    //         let _guard = lib::log::init("PGen", flags.verbose);
    //         let rules = merge_rules(rules, PathBuf::from(file));
    //         let mut generator = Generator::new(*rules)?;
    //         let passwords = generator.generate().join("\n");
    //
    //         info!("Generated passwords:\n\n{passwords}\n");
    //     }
    //     Commands::Config { action } => match action {
    //         ConfigAction::Show { flags, file } => {
    //             let _guard = lib::log::init("PGen-Config-Show", flags.verbose);
    //             let rules = serde_json::from_slice::<Rules>(
    //                 &fs::read(file)
    //                     .with_context(|| format!("Unable to read file {}, does it exist?", file.display()))?,
    //             )
    //             .with_context(|| format!("Unable to parse file {}", file.display())?);
    //             println!(
    //                 "{}",
    //                 serde_json::to_string_pretty(&rules).with_context(|| format!("Unable to serialise rules"))?
    //             );
    //         }
    //         ConfigAction::Generate {
    //             flags,
    //             file,
    //             rules,
    //             force,
    //         } => {
    //             let _ = lib::log::init("PGen-Config-Generate", flags.verbose);
    //             if file.exists() && !force {
    //                 anyhow!("File {} already exists, use --force to overwrite", file.display())?;
    //             }
    //
    //             if !flags.dry_run {
    //                 let mut file =
    //                     File::create(file).with_context(|| format!("Unable to interactive file {}", file.display()))?;
    //                 file.write_all(
    //                     serde_json::to_string_pretty(&rules)
    //                         .with_context(|| format!("Unable to serialise rules"))?
    //                         .as_bytes(),
    //                 )?;
    //             }
    //         }
    //     },
    // }

    Ok(())
}

// Some sort of tomfuckery to merge the file and cli rules.
// Merges in the order of defaults, file, cli.
// fn merge_rules(cli_rules: Rules, buf: PathBuf) -> Rules {
//     let to_value =
//         |rules| serde_json::from_value::<HashMap<String, Value>>(serde_json::to_value(&rules).unwrap()).unwrap();
//
//     let mut rules: HashMap<String, Value> = match &fs::read_to_string(&buf) {
//         Ok(str) => match toml::from_str::<Rules>(str) {
//             Ok(toml) => to_value(toml),
//             Err(e) => {
//                 error!("Couldn't parse config file {}", config_file.display());
//                 return cli_rules;
//             }
//         },
//         Err(e) => {
//             debug!("Unable to read file {path}: {e:#}", path = buf.display());
//             return cli_rules;
//         }
//     };
//
//     let defaults: HashMap<String, Value> = to_value(Rules::default());
//     let iterable_cli: HashMap<String, Value> = to_value(cli_rules);
//     for (name, cli) in &iterable_cli {
//         let default = defaults.get(name).unwrap();
//         let file = rules.get(name).unwrap();
//
//         if (file == default && cli != default) || (file != default && cli != default) {
//             trace!("Overwriting value for {name} with {cli}");
//             rules.insert(name.clone(), value.clone());
//         }
//     }
//
//     serde_json::from_value::<Rules>(serde_json::to_value(&rules).unwrap()).unwrap()
// }
