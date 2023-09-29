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

#[macro_export]
macro_rules! dyn_impl {
    (impl $trait:ty where {
        $(
            $(#[$cfg:meta])*
            for $receiver:ty where {
                $(
                    $where_ident:ident: $where_ty:tt $(+ $where_ty_add:tt)*
                ),*
            }
        ),*
    } for $impl_body:tt) => {
        $(
            $(#[$cfg])*
            impl<$($where_ident),*> $trait for $receiver where
                $($where_ident: $where_ty $(+ $where_ty_add)*),*
            $impl_body
        )*
    }
}

#[macro_export]
macro_rules! feature_trait {
    ($vis:vis trait $name:ident where {
        $(
            $(#[$meta:meta])+
            $(for $super_path:tt $(+ $super_path_add:path)*)? $(where {
                $($where_ident:ident: $where_ty:tt $(+ $where_ty_add:path)*),*
            })?
        ),+
    } for $impl_body:tt) => {
        $(
            $(#[$meta])+
            $vis trait $name$(: $super_path $(+ $super_path_add)*)? $(where $($where_ident: $where_ty $(+ $where_ty_add)*),*)?
            $impl_body
        )+
    }
}
