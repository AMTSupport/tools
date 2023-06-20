use crate::config::runtime::RuntimeConfig;
use indicatif::MultiProgress;
use lib::anyhow::Result;
use tracing::debug;
use std::path::PathBuf;

// TODO :: Implement logic from cleaner crate to handle this!
pub trait Prune {
    /// The files which should be possible to prune.
    /// The files returned by this method will be parsed,
    /// Against the `AutoPrune` struct to determine if they should be removed.
    fn files(&self, config: &RuntimeConfig) -> Vec<PathBuf>;

    /// The main prune function.
    /// This function has a common implementation for all sources,
    /// But can be overridden if needed.
    /// # Arguments
    /// * `rules` - The `AutoPrune` struct which contains the rules for pruning.
    /// # Returns
    /// A `Result` with the `Vec<PathBuf>` of the files which were removed.
    fn prune(&self, config: &RuntimeConfig, _progress_bar: &MultiProgress) -> Result<Vec<PathBuf>> {
        let files = self.files(config);
        let mut files = files.iter();
        let mut removed_files = vec![];

        // TODO :: Add dry run option.
        while let Some(file) = files.next() {
            if !(config.config.rules.auto_prune.should_prune(file, files.len())?) {
                debug!("Unable to prune file: {}", file.display());
                continue;
            }

            debug!("Pruning file: {}", file.display());
            std::fs::remove_file(file)?;
            removed_files.push(file.clone());
        }

        Ok(removed_files)
    }
}
