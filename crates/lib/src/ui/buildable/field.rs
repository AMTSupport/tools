/*
 * Copyright (c) 2023. James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use crate::ui::buildable::filler::Fillable;
use crate::ui::buildable::Builder;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct BuildableField<FieldType, BuildableType>
where
    FieldType: Sized + Debug + Into<Fillable<FieldType>>,
    BuildableType: Builder,
{
    /// The name of the field.
    pub name: &'static str,

    /// If this field is required for the final struct.
    pub required: bool,

    /// The value of the field.
    pub(crate) value: Option<FieldType>,

    pub(crate) fillable: Fillable<FieldType>,

    #[doc(hidden)]
    pub(crate) buildable: PhantomData<BuildableType>,
}

impl<FieldType, BuildableType> BuildableField<FieldType, BuildableType>
where
    FieldType: Sized + Debug + Into<Fillable<FieldType>>,
    BuildableType: Builder,
{
    pub fn new(name: &'static str, required: bool) -> Self {
        Self {
            name,
            required,
            value: None,
            fillable: PhantomData::<FieldType>.into(),
            buildable: PhantomData::<BuildableType>,
        }
    }

    /// Returns the value of the field if it exists.
    ///
    /// If the field does not exist, this will return None.
    /// If the field exists but is not the correct type, this will return None.
    /// If the field exists, is the correct type and has a value, this will return Some(value).
    pub fn get(&self) -> Option<&FieldType> {
        self.value.as_ref()
    }
}

// impl<F, FT, FF> !FromStr for dyn Builder<Field = F, FieldType = FT, Final = FF> where Self: Sized {}

// impl<F, B> BuildableFiller for BuildableField<F, B>
// where
//     F: Buildable,
//     B: Builder,
// {
//     async fn fill<BuildableStruct>(&self) -> Result<<BuildableStruct as Builder>::Final>
//     where
//         BuildableStruct: Builder,
//     {
//     }
// }
//
// impl<FieldType, BuildableStruct> BuildableField<FieldType, BuildableStruct>
// where
//     BuildableStruct: Builder,
//     FieldType: Sized + Debug,
// {
//     /// Attempts to set the field from a string.
//     ///
//     /// If this field cannot be parsed using [`FromStr`]
//     /// it will return an Err result of the parsing error.
//     ///
//     /// If there was an existing value on this field and parsing was successful,
//     /// the object which was the previous value will be returned in an [`Option`].
//     /// If the type of [`T`] is not the correct type of the requested field,
//     /// an Err result will be returned.
//     pub async fn fill<Filler>(&mut self) -> Result<Option<FieldType>>
//     where
//         Filler: BuildableFiller,
//     {
//         let fillable = Fillable::from(PhantomData::<FieldType>);
//
//         Ok(None)
//     }
//
//     /// Returns the value of the field if it exists.
//     ///
//     /// If the field does not exist, this will return None.
//     /// If the field exists but is not the correct type, this will return None.
//     /// If the field exists, is the correct type and has a value, this will return Some(value).
//     pub fn get(&self) -> Option<&FieldType> {
//         self.value.as_ref()
//     }
// }
