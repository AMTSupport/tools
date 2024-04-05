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

#[macro_export]
macro_rules! impls_utils {
    ($($async:ident fn)? $([$($(+)? $common_impl:ty)+])? |$phantom_data:ident $(, $($variable:ident: $variable_type:ty),+)?| $impl_target:path => $return_type:ty $(| $alt_wrapper_return_type:ty)? => $f:expr) => {{
        trait DoesNotImpl<T: Sized + Clone + Debug> {
            $($async)? fn call(&self$(, $($variable: $variable_type),+)?) -> $crate::ui::builder::error::FillResult<$return_type>;
        }

        impl<B: Debug + Sized, T: Sized + Clone + Debug> DoesNotImpl<T> for B {
            #[allow(unused)]
            // #[tracing::instrument(level = "TRACE", err, ret)]
            $($async)? fn call(&self$(, $(paste::paste! { [< _ $variable >] }: $variable_type),+)?) -> $crate::ui::builder::error::FillResult<$return_type> {
                Err($crate::ui::builder::error::FillError::InvalidFiller { field: "blah".to_string(), filler: stringify!($impl_target).to_string() })
            }
        }

        #[derive(Debug)]
        struct Wrapper<T: Sized + Clone + Debug $($(+ $common_impl)+)?>(PhantomData<T>);
        impl<T: Sized + Clone + Debug + $impl_target $($(+ $common_impl)+)?> Wrapper<T> {
            #[allow(unused)]
            // #[tracing::instrument(level = "TRACE", err, ret)]
            $($async)? fn call(&self$(, $($variable: $variable_type),+)?) -> $crate::impls_utils!(@return $return_type $(| $alt_wrapper_return_type)?) {return $f;}
        }

        let wrapper = Wrapper($phantom_data);
        $crate::impls_utils!(@wrapper_call $($async)? wrapper.call($($($variable),+)?))
    }};

    // Adds support for using an alternate return type for the actual wrapper function.
    (@return $return_type:ty) => {
        $crate::ui::builder::error::FillResult<$return_type>
    };
    (@return $return_type:ty | $alt_return_type:ty) => {
        $crate::ui::builder::error::FillResult<$alt_return_type>
    };

    // This is a hack to allow the macro to work with and without the async keyword.
    (@wrapper_call async $wrapper:ident.call($($($variable:ident),+)?)) => {
        $wrapper.call($($($variable),+)?).await
    };
    (@wrapper_call $wrapper:ident.call($($($variable:ident),+)?)) => {
        $wrapper.call($($($variable),+)?)
    };
}

// #[instrument(level = "TRACE", err, ret)]
// pub async fn try_fill<F: Filler, T: Sized + Clone + Debug>(wrapped: &TypeWrapped<T>, filler: &mut F) -> FillResult<T> {
//     match wrapped {
//         TypeWrapped::Bool(def) => {
//             debug!("filling bool {}", def.name);
//
//             filler.fill_bool(def).await.map(|v| unsafe { transmute_unchecked(v) })
//         }
//         TypeWrapped::String(def) => {
//             debug!("filling string {}", def.name);
//
//             let pd = def._pd;
//             impls_utils!(async fn |pd, filler: &mut impl Filler, def: &FillableDefinition<T>| FromStr => T => {
//                 filler.fill_input(def).await.map(|v| {
//                     unsafe { transmute_unchecked(v) }
//                 })
//             })
//         }
//         TypeWrapped::Buildable(def) => {
//             debug!("filling buildable {}", def.name);
//
//             let pd = def._pd;
//             impls_utils!(async fn |pd, filler: &mut impl Filler| Buildable => T | T::Builder => {
//                 use crate::ui::builder::Builder;
//
//                 let mut builder = T::builder();
//                 builder.fill(filler).await?;
//                 Ok(builder)
//             })
//         }
//     }
// }
