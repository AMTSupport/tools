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

use anyhow::Result;
use lib::ui::cli::cli::{AsyncCliUI, CliUI};
use memorable_pass::ui::cli::ui::MemorablePassCli;

#[tokio::main]
async fn main() -> Result<()> {
    let mut cli = MemorablePassCli::new(())?;
    cli.run().await?;

    // match cli.command {
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
