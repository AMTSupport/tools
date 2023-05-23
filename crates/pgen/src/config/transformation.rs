use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum,
)]
pub enum Transformation {
    None,
    Capitalise,
    AllExcludingFirst,
    Uppercase,
    Random,
    Alternating,
}

impl Transformation {
    /// Transforms a string with the given transformation.
    /// This is an inplace transformation and will mutate the string.
    /// Returns the new pointer to the string.
    pub fn transform(&self, mut str: String) -> String {
        match self {
            Transformation::None => return str,
            Transformation::Uppercase => str.make_ascii_uppercase(),
            Transformation::Capitalise => str.chars().next().unwrap().make_ascii_uppercase(),
            Transformation::AllExcludingFirst => {
                str.replace_range(1..str.len(), &str[1..2].to_uppercase())
            }
            Transformation::Alternating => {
                let mut chars = str.chars();
                for i in 0..str.len() {
                    if &i % 2 != 0 {
                        continue;
                    }

                    chars
                        .nth((i / 2).clamp(0, 9))
                        .unwrap()
                        .make_ascii_uppercase();
                }
            }
            Transformation::Random => {
                use rand::distributions::{Bernoulli, Distribution};

                let mut rng = rand::thread_rng();
                let bernoulli = Bernoulli::new(0.5).unwrap();
                let mut chars = str.chars().collect::<Vec<char>>();

                for i in 0..str.len() {
                    if bernoulli.sample(&mut rng) {
                        continue;
                    }

                    unsafe {
                        chars.get_unchecked_mut(i).make_ascii_uppercase();
                    }
                }
            }
        }

        str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformations() {
        let str = String::from("watermelon");
        Transformation::None.transform(str.clone());
        assert_eq!(str, "watermelon");

        Transformation::Capitalise.transform(str.clone());
        assert_eq!(str, "Watermelon");

        Transformation::AllExcludingFirst.transform(str.clone());
        assert_eq!(str, "wATERMELON");

        Transformation::Uppercase.transform(str.clone());
        assert_eq!(str, "WATERMELON");

        Transformation::Random.transform(str.clone());
        assert_ne!(str, "watermelon");

        Transformation::Alternating.transform(str.clone());
        assert_eq!(str, "WaTeRmElOn");
    }
}
