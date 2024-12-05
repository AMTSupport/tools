/*
 * Copyright (C) 2024. James Draycott me@racci.dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

use crate::conditional_call;
use crate::error::{FillError, FillResult};
use amt_lib::feature_trait;
use amt_lib::ui::cli::ui_inquire::STYLE;
use amt_lib::ui::cli::CliUi;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub struct FillableDefinition<T>
where
    T: Sized + Clone,
{
    pub name: &'static str,
    pub required: bool,
    pub default: Option<T>,
    pub description: Option<&'static str>,

    pub value: Option<T>,

    pub _pd: PhantomData<T>,
}

pub trait Filler
where
    Self: Debug,
{
    async fn fill_bool(&self, fillable: &FillableDefinition<bool>) -> FillResult<bool>;

    // async fn fill_choice<T>(&mut self, fillable: Fillable<T>, items: Vec<T>, default: Option<T>) -> FillResult<T>;

    async fn fill_input<T>(&self, fillable: &FillableDefinition<T>) -> FillResult<T>
    where
        T: FromStr + Clone + Debug;
}

feature_trait! {
    impl Filler for C where {
        #[cfg(feature = "repl")]
        where {
            O: Sized,
            R: Sized,
            C: CliUi<OneshotAction = O, ReplAction = R>
        },
        #[cfg(all(not(feature = "repl")))]
        where {
            O: Sized,
            C: CliUi<OneshotAction = O>
        }
    } for {
        async fn fill_bool(&self, fillable: &FillableDefinition<bool>) -> FillResult<bool> {
            let message = format!("enter a value for {}", fillable.name);
            let mut prompt = inquire::Confirm::new(&message);
            prompt.help_message = fillable.description;
            prompt.default = fillable.default;

            prompt
                .with_render_config(*STYLE)
                .with_placeholder("y/n")
                .with_error_message("invalid input, please enter y/n")
                .prompt()
                .map_err(|e| FillError::InvalidInput {
                    field: fillable.name.to_string(),
                    input: e.to_string(),
                })
        }

        // async fn fill_choice<T>(&mut self, fillable: Fillable<T>, items: Vec<T>, default: Option<T>) -> FillResult<T> {
        //     let (items, default) = if fillable.can_display {
        //         (unsafe { transmute_unchecked::<_, Vec<Box<dyn Display>>>(items) },
        //         default.map(|d| unsafe { transmute_unchecked::<_, impl Display>(d) }))
        //     } else {
        //         struct Wrapper<T>(T);
        //         impl <T> Display for Wrapper<T> {
        //             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //                 write!(f, "Non-Display Type {}", type_name::<T>())
        //             }
        //         }
        //
        //         (items.into_iter().map(|i| Wrapper(i)).collect(), default.map(|d| Wrapper(d)))
        //     };
        //
        //     inquire::Select::new(&*format!("enter a value for {}", fillable.name), items)
        //         .prompt()
        //         .map_err(|e| FillError::InvalidInput {
        //             field: fillable.name.to_string(),
        //             input: e.to_string(),
        //         })
        //         .into()
        // }

    async fn fill_input<T>(&self, fillable: &FillableDefinition<T>) -> FillResult<T>
        where
            T: FromStr + Clone + Debug,
        {
            let message = format!("enter a value for {}", fillable.name);
            let mut prompt = inquire::Text::new(&message);
            prompt.help_message = fillable.description;
            prompt.placeholder = fillable.description;
            let default_or_none = fillable
                .default
                .as_ref()
                .map(|d| {
                    conditional_call! {
                        impl T where T: Sized | T: ToString {
                            fn call(d: &T) -> Option<String> {
                                Some(d.to_string())
                            } else {
                                None
                            }
                        }
                    };

                    conditional_call!(call::<T>(d))
                })
                .flatten();
            prompt.default = default_or_none.as_deref();

            prompt
                .with_render_config(*STYLE)
                .prompt()
                .map_err(|e| FillError::InvalidInput {
                    field: fillable.name.to_string(),
                    input: e.to_string(),
                })
                .and_then(|s| {
                    s.parse().map_err(|_| FillError::InvalidInput {
                        field: fillable.name.to_string(),
                        input: s.to_string(),
                    })
                })
        }
    }
}

// #[cfg(feature = "cli")]
// impl<T, O, R> Filler for T
// where
//     T: CliUi<OneshotAction = O, ReplAction = R>,
// {
//     async fn fill_bool(&self, fillable: &FillableDefinition<bool>) -> FillResult<bool> {
//         let message = format!("enter a value for {}", fillable.name);
//         let mut prompt = inquire::Confirm::new(&message);
//         prompt.help_message = fillable.description;
//         prompt.default = fillable.default;
//
//         prompt
//             .with_render_config(*STYLE)
//             .with_placeholder("y/n")
//             .with_error_message("invalid input, please enter y/n")
//             .prompt()
//             .map_err(|e| FillError::InvalidInput {
//                 field: fillable.name.to_string(),
//                 input: e.to_string(),
//             })
//     }
//
//     // async fn fill_choice<T>(&mut self, fillable: Fillable<T>, items: Vec<T>, default: Option<T>) -> FillResult<T> {
//     //     let (items, default) = if fillable.can_display {
//     //         (unsafe { transmute_unchecked::<_, Vec<Box<dyn Display>>>(items) },
//     //         default.map(|d| unsafe { transmute_unchecked::<_, impl Display>(d) }))
//     //     } else {
//     //         struct Wrapper<T>(T);
//     //         impl <T> Display for Wrapper<T> {
//     //             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     //                 write!(f, "Non-Display Type {}", type_name::<T>())
//     //             }
//     //         }
//     //
//     //         (items.into_iter().map(|i| Wrapper(i)).collect(), default.map(|d| Wrapper(d)))
//     //     };
//     //
//     //     inquire::Select::new(&*format!("enter a value for {}", fillable.name), items)
//     //         .prompt()
//     //         .map_err(|e| FillError::InvalidInput {
//     //             field: fillable.name.to_string(),
//     //             input: e.to_string(),
//     //         })
//     //         .into()
//     // }
//
//     async fn fill_input<T>(&self, fillable: &FillableDefinition<T>) -> FillResult<T>
//     where
//         T: FromStr + Clone + Debug,
//     {
//         let message = format!("enter a value for {}", fillable.name);
//         let mut prompt = inquire::Text::new(&message);
//         prompt.help_message = fillable.description;
//         prompt.placeholder = fillable.description;
//         let default_or_none = fillable
//             .default
//             .as_ref()
//             .map(|d| {
//                 conditional_call! {
//                     impl T where T: Sized | T: ToString {
//                         fn call(d: &T) -> Option<String> {
//                             Some(d.to_string())
//                         } else {
//                             None
//                         }
//                     }
//                 };
//
//                 conditional_call!(call::<T>(d))
//             })
//             .flatten();
//         prompt.default = default_or_none.as_deref();
//
//         prompt
//             .with_render_config(*STYLE)
//             .prompt()
//             .map_err(|e| FillError::InvalidInput {
//                 field: fillable.name.to_string(),
//                 input: e.to_string(),
//             })
//             .and_then(|s| {
//                 s.parse().map_err(|_| FillError::InvalidInput {
//                     field: fillable.name.to_string(),
//                     input: s.to_string(),
//                 })
//             })
//     }
// }
