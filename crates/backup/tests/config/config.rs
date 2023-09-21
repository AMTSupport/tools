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

// mod find {
//     use super::*;
//
//     #[test_log::test(test)]
//     fn find_error() -> Result<()> {
//         let temp = TempDir::new()?;
//
//         assert_matches!(
//             Config::find(Some(&temp.path())),
//             Err(config::Error::Find),
//             "Expected Config::find to find no possible config files"
//         );
//         Ok(())
//     }
//
//     #[test_log::test(test)]
//     fn find_env() -> Result<()> {
//         let file = assert_fs::NamedTempFile::new("settings.json")?;
//         env::set_var(Config::ENV_VAR, file.path());
//
//         assert_matches!(
//             Config::find(None),
//             Ok(path) if path == file.path(),
//             "Expected Config::find to find the config file from the environment variable"
//         );
//
//         Ok(())
//     }
//
//     #[test_log::test(test)]
//     fn find_path() -> Result<()> {
//         let temp = TempDir::new()?;
//         let child = temp.child("settings.json");
//         child.touch()?;
//
//         assert_matches!(
//             Config::find(Some(&temp.path())),
//             Ok(path) if path == path,
//             "Expected Config::find to find the config file from the path"
//         );
//
//         Ok(())
//     }
//
//     #[test_log::test(test)]
//     fn find_current() -> Result<()> {
//         let temp = TempDir::new()?;
//         let child = temp.child("settings.json");
//         child.touch()?;
//
//         env::set_current_dir(&temp.path())?;
//
//         assert_matches!(
//             Config::find(None),
//             Ok(path) if path == child.path(),
//             "Expected Config::find to find the config file from the current directory"
//         );
//
//         Ok(())
//     }
// }
//
// #[test]
// fn test_save() {
//     let temp = TempDir::new().unwrap();
//     let config = Config::new(&temp.path());
//
//     assert!(config.save().is_ok());
//     assert!(config.path.unwrap().exists());
// }
//
// #[test]
// fn test_save_no_path() {
//     let temp = TempDir::new().unwrap();
//     let mut config = Config::new(&temp.path());
//     config.path = None;
//
//     assert!(config.save().is_err());
// }
//
// #[test]
// fn test_save_no_parent() {
//     let temp = TempDir::new().unwrap();
//     let mut config = Config::new(&temp.path());
//     config.path = Some(temp.path().join("test").join("settings.json"));
//     assert!(config.save().is_err());
// }
//
// #[test]
// fn test_load() {
//     let temp = TempDir::new().unwrap();
//     let config = Config::new(&temp.path());
//
//     assert!(config.save().is_ok());
//     assert!(config.path.unwrap().exists());
//
//     let loaded = Config::load(&config.path.unwrap()).unwrap();
//     assert_eq!(config, loaded);
// }
//
// #[test]
// fn test_load_no_file() {
//     let temp = TempDir::new().unwrap();
//     let config = Config::new(&temp.path());
//
//     assert!(config.path.unwrap().exists());
//     assert!(config.load().is_err());
// }
//
// #[test]
// fn test_load_invalid_file() {
//     let temp = TempDir::new().unwrap();
//     let config = Config::new(&temp.path());
//
//     assert!(config.save().is_ok());
//     assert!(config.path.unwrap().exists());
//
//     let mut file = fs::File::create(&config.path.unwrap()).unwrap();
//     file.write_all(b"invalid").unwrap();
//
//     assert!(config.load().is_err());
// }
