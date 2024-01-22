/*
 * Copyright (c) 2023-2024. James Draycott <me@racci.dev>
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
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use crate::ui::builder::buildable::Buildable;
use crate::ui::builder::error::{FillError, FillResult};
use crate::ui::builder::filler::{FillableDefinition, Filler, TypeWrapped, TypeWrappedRet};
use crate::ui::builder::Builder;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::transmute;
use std::str::FromStr;
use tracing::debug;

// trait MaybeImplements<Trait: ?Sized, Ret> {
//     fn call(&self) -> Option<Ret>;
// }
//
// impl<T: Buildable, B: Builder> MaybeImplements<dyn Buildable<Builder = B>, B> for T {
//     fn call(&self) -> Option<B> {
//         Some(T::builder())
//     }
// }
//
// auto trait NotBuildable {}
// impl<T> !NotBuildable for T where T: Buildable {}
//
// impl<T, B> MaybeImplements<dyn Buildable<Builder = B>, B> for T
// where
//     T: NotBuildable,
//     B: Buildable,
// {
//     fn call(&self) -> Option<B> {
//         None
//     }
// }

trait DoesNotImpl<T> {
    fn builder(&self) -> Option<T>;
}

impl<T: Sized, B: Sized> DoesNotImpl<B> for T {
    fn builder(&self) -> Option<B> {
        None
    }
}

struct Wrapper<T: Sized>(PhantomData<T>);
impl<T: Buildable> Wrapper<T> {
    fn builder(&self) -> Option<<T as Buildable>::Builder> {
        Some(T::builder())
    }
}

macro_rules! impls {
    ($([$($var:ident: $ty:ty),+])? $pd:ident > $v:path > $r:ty => $f:expr) => {{
        trait DoesNotImpl<T: Sized> {
            async fn call(&self$(, $($var: $ty),+)?) -> Option<T>;
        }

        impl<T: Sized, B: Sized> DoesNotImpl<B> for T {
            async fn call(&self$(, $($var: $ty),+)?) -> Option<B> {
                None
            }
        }

        struct Wrapper<T: Sized>(PhantomData<T>);
        impl<T: $v> Wrapper<T> {
            async fn call(&self$(, $($var: $ty),+)?) -> Option<$r> {$f}
        }

        let wrapper = Wrapper($pd);
        return if let Some(value) = wrapper.call($(, $($var),+)?).await {
            Ok(value);
        } else {
            tracing::warn!("filler for {} returned None", stringify!($v));
            Err(FillError::Unknown(Box::new("filler returned None")))
        }
    }};
}

// macro_rules! impls_ret {
//     ([$fillable:ident, $filler:ident] $t:ty: $v:path => $f:expr) => {
//         let opt = async {
//             trait DoesNotImpl {
//                 #[tracing::instrument(level = "TRACE", skip(_fillable, _filler))]
//                 async fn call<F: Filler>(_fillable: Fillable<Self>, _filler: &mut F) -> FillResult<Option<Self>> where Self: Sized {
//                     Ok(None)
//                 }
//             }
//
//             impl<T: Sized> DoesNotImpl for T {}
//
//             struct Wrapper<T: Sized>(std::marker::PhantomData<T>);
//
//             impl<Trans: $v> Wrapper<Trans> {
//                 #[tracing::instrument(level = "TRACE", skip(fillable, filler))]
//                 async fn call<F: Filler>(fillable: Fillable<Trans>, filler: &mut F) -> FillResult<Option<Trans>> {
//                     let value = {$f}(fillable, filler).await?;
//                     Ok(Some(unsafe { std::intrinsics::transmute_unchecked::<_, Trans>(value) }))
//                 }
//             }
//
//             if $fillable.default.is_some() {
//                 tracing::warn!("default value is ignored currently.");
//             }
//
//             <Wrapper<$t>>::call(unsafe { std::mem::transmute_copy(&$fillable) }, $filler).await
//         }.await?;
//
//         if let Some(value) = opt {
//             // After unwrapping the result and making sure its not None, we can safely transmute it back into the original type.
//             return Ok(unsafe { transmute_unchecked::<_, T>(value) });
//         } else {
//             tracing::warn!("filler for {} returned None", $fillable.name);
//         }
//     };
// }

pub async fn try_fill<F: Filler, T>(wrapped: TypeWrapped<T>, filler: &mut F) -> FillResult<TypeWrappedRet<T>> {
    /// Transmute the Src into Dst, invoke the function and then transmute the Dst back into Src.
    // async fn transmute_invoke_return<O, T, R, F>(
    //     fillable: Fillable<O>,
    //     f: impl FnOnce(Fillable<T>) -> F,
    // ) -> FillResult<O>
    // where
    //     F: Future<Output = FillResult<R>>,
    // {
    //     // let fillable = Fillable::<T> {
    //     //     default: fillable.default.map(|v| unsafe { transmute_unchecked(v) }),
    //     //     ..fillable.clone()
    //     // };
    //
    //     let trans = unsafe { std::mem::transmute(fillable) };
    //     let ret = f(trans).await?;
    //     Ok(unsafe { transmute_unchecked::<R, O>(ret) })
    // }
    match wrapped {
        TypeWrapped::Bool(def) => {
            debug!("filling bool {}", def.name);
            filler.fill_bool(def).await.map(|v| TypeWrappedRet::Bool(unsafe { transmute(v) }))
        }
        TypeWrapped::String { def, .. } => {
            debug!("filling string {}", def.name);
            impls!([filler: F, def: FillableDefinition<T>] FromStr > TypeWrappedRet<T> => {
                let value = filler.fill_input(def, PhantomData::<T>).await?;
                Some(TypeWrappedRet::String(unsafe { transmute(value) }, PhantomData::<T>))
            });
        }
        TypeWrapped::Buildable { pd, def } => {
            debug!("filling buildable {}", def.name);

            let wrapper = Wrapper(pd);
            let builder = wrapper.builder().unwrap();
            // let builder = pd.call().ok_or(FillError::Unknown(Box::new("failed to get builder")))?;
            let builder = builder.fill(filler).await?;

            match builder.build().await {
                Ok(v) => Ok(TypeWrappedRet::Buildable(v, pd)),
                Err(e) => Err(FillError::Unknown(Box::new(e))),
            }
        }
    }

    // match fillable.pure_type {
    //     PureType::Bool => {
    //         debug!("filling bool {}", fillable.name);
    //         return transmute_invoke_return(fillable, |fillable| filler.fill_bool(fillable)).await;
    //     }
    //     PureType::FromStr => {
    //         debug!("filling string {}", fillable.name);
    //
    //         impls_ret!([fillable, filler] T: FromStr => async move |fillable, filler: &mut F| {
    //             filler.fill_input(fillable).await
    //         });
    //     }
    //     PureType::Buildable => {
    //         debug!("filling buildable {}", fillable.name);
    //
    //         impls_ret!([fillable, filler] T: Buildable => async move |_, filler: &mut F| {
    //             let builder = Trans::builder();
    //             let builder = builder.fill(filler).await?;
    //
    //             builder.build().await.map_err(|e| FillError::Unknown(Box::new(e)))
    //         });
    //     }
    // };
}
