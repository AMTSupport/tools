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

#![feature(associated_type_defaults)]
#![feature(async_closure)]
#![feature(auto_traits)]
#![feature(cfg_match)]
#![feature(const_for)]
#![feature(const_format_args)]
#![feature(core_intrinsics)]
#![feature(fn_traits)]
#![feature(impl_trait_in_assoc_type)]
#![feature(let_chains)]
#![feature(negative_impls)]
#![feature(result_flattening)]
#![feature(stmt_expr_attributes)]
#![feature(trait_alias)]
#![feature(trivial_bounds)]
#![feature(try_trait_v2)]
#![feature(type_alias_impl_trait)]
#![feature(negative_bounds)]
#![feature(lazy_type_alias)]
#![allow(async_fn_in_trait)]
#![allow(internal_features)]
#![allow(incomplete_features)]

#[doc(hidden)]
pub extern crate paste as _paste;

pub mod fs;
pub mod helper;
pub mod log;
pub mod macros;
pub mod named;
pub mod pathed;
#[cfg(any(feature = "ui-gui", feature = "ui-tui", feature = "ui-cli"))]
pub mod ui;
#[cfg(feature = "updater")]
pub mod updater;

#[macro_export]
macro_rules! conditional_call {
    (
        impl $call_type:ty
        where $($generic_type_ident:ident: $first_generic_type_ty:tt $(+ $generic_type_ty:tt)*),+ // The types for the required parameters
        | $($top_generic_ident:ident: $first_top_generic_ty:tt $(+ $top_generic_ty:tt)*),+ // The types for the trait / struct
        {
            async fn call // Whether the function is async or not
            $(<$($($generic_life:lifetime),+ $(,)?)? $($($generic_ident:ident: $first_generic_ty:tt $(+ $generic_ty:tt)*),+)?>)? // The types for the function
            ($($variable:ident: $variable_type:ty),*) // The parameters of the function
                -> $target_type:ty // The return type of the function
                $func_body:block // The body of the function if the actual call is possible
            else $not_call_body:block // The body of the function if the actual call isn't possible
        }
    ) => {
        $crate::conditional_call! {@internal
            impl $call_type
            where $($generic_type_ident: $first_generic_type_ty $(+ $generic_type_ty)*),+
            | $($top_generic_ident: $first_top_generic_ty $(+ $top_generic_ty)*),+
            {
                async fn | call
                $(<$($($generic_life),* ,)? $($($generic_ident: $first_generic_ty $(+ $generic_ty)*),*)?>)?
                ($($variable: $variable_type),*) -> $target_type
                $func_body
                else $not_call_body
            }
        }
    };

    (
        impl $call_type:ty
        where $($generic_type_ident:ident: $first_generic_type_ty:tt $(+ $generic_type_ty:tt)*),+ // The types required for the parameters
        | $($top_generic_ident:ident: $first_top_generic_ty:tt $(+ $top_generic_ty:tt)*),+ // The types for the trait / struct
        {
            fn call // Whether the function is async or not
            $(<$($($generic_life:lifetime),+ $(,)?)? $($($generic_ident:ident: $first_generic_ty:tt $(+ $generic_ty:tt)*),+)?>)? // The types for the function
            ($($variable:ident: $variable_type:ty),*) // The parameters of the function
                -> $target_type:ty // The return type of the function
                $func_body:block // The body of the function if the actual call is possible
            else $not_call_body:block // The body of the function if the actual call isn't possible
        }
    ) => {
        $crate::conditional_call! {@internal
            impl $call_type
            where $($generic_type_ident: $first_generic_type_ty $(+ $generic_type_ty)*),+
            | $($top_generic_ident: $first_top_generic_ty $(+ $top_generic_ty)*),+
            {
                fn | call
                $(<$($($generic_life),* ,)? $($($generic_ident: $first_generic_ty $(+ $generic_ty)*),*)?>)?
                ($($variable: $variable_type),*) -> $target_type
                $func_body
                else $not_call_body
            }
        }
    };

    (call::<$($type:ty),+>($($parameter:expr),*)) => {
        <Wrapper<$($type),+>>::call($($parameter),*)
    };

    (@internal
        impl $call_type:ty
        where $($generic_type_ident:ident: $first_generic_type_ty:tt $(+ $generic_type_ty:tt)*),+ // The types required for the parameters
        | $($top_generic_ident:ident: $first_top_generic_ty:tt $(+ $top_generic_ty:tt)*),+ // The types for the trait / struct
        {
            $($func_fn:ident )+ | call // Whether the function is async or not
            $(<$($($generic_life:lifetime),+ $(,)?)? $($($generic_ident:ident: $first_generic_ty:tt $(+ $generic_ty:tt)*),+)?>)? // The types for the function
            ($($variable:ident: $variable_type:ty),*) // The parameters of the function
                -> $target_type:ty // The return type of the function
                $func_body:block // The body of the function if the actual call is possible
            else $not_call_body:block // The body of the function if the actual call isn't possible
        }
    ) => {
        trait NotCall<$($generic_type_ident: $first_generic_type_ty $(+ $generic_type_ty)*),+> {
            $($func_fn )+ call
            $(<$($($generic_life),* ,)? $($($generic_ident: $first_generic_ty $(+ $generic_ty)*),*)?>)?
            ($($variable: $variable_type),*) -> $target_type;
        }

        impl<$($generic_type_ident: $first_generic_type_ty $(+ $generic_type_ty)*),+> NotCall<$($generic_type_ident),+> for Wrapper<$($generic_type_ident),+> {
            #[allow(unused)]
            $($func_fn )+ call
            $(<$($($generic_life),* ,)? $($($generic_ident: $first_generic_ty $(+ $generic_ty)*),*)?>)?
            ($($variable: $variable_type),*) -> $target_type { $not_call_body }
        }

        struct Wrapper<$($generic_type_ident: $first_generic_type_ty $(+ $generic_type_ty)*),+>($(core::marker::PhantomData<$generic_type_ident>),+);

        #[allow(dead_code)]
        impl <$($top_generic_ident: $first_top_generic_ty $(+ $top_generic_ty)*),+> Wrapper<$($top_generic_ident),+> {
            $($func_fn )+ call
            $(<$($($generic_life),* ,)? $($($generic_ident: $first_generic_ty $(+ $generic_ty)*),*)?>)?
            ($($variable: $variable_type),*) -> $target_type $func_body
        }
    }
}
