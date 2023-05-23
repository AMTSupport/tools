use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum
)]
pub enum SeparatorMode {
    None,
    Random,
}

pub(crate) const CHARS: [char; 12] = ['!', '@', '$', '%', '.', '&', '*', '-', '+', '=', '?', ':'];

impl SeparatorMode {
    pub fn get(&self, chars: &[char; 12]) -> char {
        match self {
            SeparatorMode::None => '\0',
            SeparatorMode::Random => {
                use rand::distributions::{Distribution, Uniform};

                let mut rng = rand::thread_rng();
                let uniform = Uniform::new(0, CHARS.len());

                chars.get(uniform.sample(&mut rng)).unwrap().clone()
            }
        }
    }
}
