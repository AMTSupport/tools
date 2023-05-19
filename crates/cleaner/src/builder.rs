use crate::{CleanablePath, PathCollections};
use anyhow::{anyhow, Context};
use chrono::Duration;
use log::trace;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::slice::Iter;

#[derive(Debug, Clone)]
pub enum AgeType {
    FromCreation,
    FromModification,
    FromAccess,
}

pub trait CleanableBuilderTrait {
    fn collection(collection: PathCollections) -> Self;
    fn env(env: &str) -> Self;
    fn path(self, composing: &str) -> Self;
    fn auto(self) -> Self;
    fn minimum_age(self, duration: Duration) -> Self;
    fn duration_from(self, age_type: AgeType) -> Self;
    fn build(&self) -> anyhow::Result<CleanablePath>;
}

#[derive(Debug, Clone)]
pub struct CleanableBuilder {
    collection: Option<PathCollections>,
    env: Option<String>,
    pub composing: Option<String>,
    auto: bool,
    minimum_age: Duration,
    duration_from: AgeType,
}

impl CleanableBuilderTrait for CleanableBuilder {
    fn collection(collection: PathCollections) -> Self {
        Self {
            collection: Some(collection),
            env: None,
            composing: None,
            auto: false,
            minimum_age: Duration::zero(),
            duration_from: AgeType::FromCreation,
        }
    }

    fn env(env: &str) -> Self {
        Self {
            collection: None,
            env: Some(env.to_string()),
            composing: None,
            auto: false,
            minimum_age: Duration::zero(),
            duration_from: AgeType::FromCreation,
        }
    }

    fn path(mut self, composing: &str) -> Self {
        self.composing = Some(composing.to_string());
        self
    }

    fn auto(mut self) -> Self {
        self.auto = true;
        self
    }

    fn minimum_age(mut self, duration: Duration) -> Self {
        self.minimum_age = duration;
        self
    }

    fn duration_from(mut self, age_type: AgeType) -> Self {
        self.duration_from = age_type;
        self
    }

    fn build(&self) -> anyhow::Result<CleanablePath> {
        let buf = PathBuf::from(self.composing.clone().context("Unwrap path")?);
        let composed = match self.collection.clone() {
            Some(collection) => match collection {
                PathCollections::Drive => drive(&buf),
                PathCollections::User => user(&buf),
            },
            None => match self.env.clone() {
                Some(env_value) => env(&env_value, &buf),
                None => Err(anyhow::anyhow!("No collection or env set")),
            },
        }?;

        Ok(CleanablePath {
            paths: composed,
            auto: self.auto,
            minimum_age: self.minimum_age,
            duration_from: self.duration_from.clone(),
        })
    }
}

fn composing(buf: &PathBuf, roots: Iter<'_, PathBuf>) -> anyhow::Result<Vec<PathBuf>> {
    let mut buffers = vec![];

    for root in roots {
        let mut root = root.clone();
        root.push(buf);
        match root.exists() {
            true => buffers.push(root),
            false => {
                trace!("Path does not exist: {}", buf.display());
                continue;
            }
        }
    }

    if buffers.is_empty() {
        return Err(anyhow!("No paths found for {}", buf.display()));
    }

    Ok(buffers)
}

fn env(variable: &str, buf: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let env_value = std::env::var(variable).context("Get env variable")?;
    let env_value_buf = PathBuf::from(env_value);

    if !env_value_buf.exists() {
        return Err(anyhow!("Env variable {} does not exist as path", variable));
    }

    return composing(buf, vec![env_value_buf].iter());
}

fn user(buf: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    #[cfg(windows)]
    let users_dir =
        PathBuf::from(std::env::var("SystemDrive").context("Get system drive.")?).join("Users");
    #[cfg(unix)]
    let users_dir = PathBuf::from("/home");

    let users = match users_dir.read_dir() {
        Ok(users) => users
            .map(|user| user.unwrap().path())
            .collect::<Vec<PathBuf>>(),
        Err(e) => return Err(anyhow!("Error while collecting users: {}", e)),
    };

    return composing(buf, users.iter());
}

#[cfg(windows)]
static DRIVES: Lazy<Vec<PathBuf>> = Lazy::new(|| {
    let mut drives = Vec::with_capacity(26);
    for x in 0..26 {
        let drive = format!("{}:", (x + 64) as u8 as char);
        let drive = PathBuf::from(drive);
        if drive.exists() {
            drives.push(drive);
        }
    }

    return drives;
});
#[cfg(unix)]
static DRIVES: Lazy<Vec<PathBuf>> = Lazy::new(|| vec![PathBuf::from("/")]);
fn drive(buf: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    return composing(buf, DRIVES.iter());
}
