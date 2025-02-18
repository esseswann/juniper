use std::{
    mem::{self, MaybeUninit},
    ptr,
};

use crate::{
    ast::{FromInputValue, InputValue, Selection, ToInputValue},
    executor::{ExecutionResult, Executor, Registry},
    schema::meta::MetaType,
    types::{
        async_await::GraphQLValueAsync,
        base::{GraphQLType, GraphQLValue},
    },
    value::{ScalarValue, Value},
};

impl<S, T> GraphQLType<S> for Option<T>
where
    T: GraphQLType<S>,
    S: ScalarValue,
{
    fn name(_: &Self::TypeInfo) -> Option<&'static str> {
        None
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        registry.build_nullable_type::<T>(info).into_meta()
    }
}

impl<S, T> GraphQLValue<S> for Option<T>
where
    S: ScalarValue,
    T: GraphQLValue<S>,
{
    type Context = T::Context;
    type TypeInfo = T::TypeInfo;

    fn type_name(&self, _: &Self::TypeInfo) -> Option<&'static str> {
        None
    }

    fn resolve(
        &self,
        info: &Self::TypeInfo,
        _: Option<&[Selection<S>]>,
        executor: &Executor<Self::Context, S>,
    ) -> ExecutionResult<S> {
        match *self {
            Some(ref obj) => executor.resolve(info, obj),
            None => Ok(Value::null()),
        }
    }
}

impl<S, T> GraphQLValueAsync<S> for Option<T>
where
    T: GraphQLValueAsync<S>,
    T::TypeInfo: Sync,
    T::Context: Sync,
    S: ScalarValue + Send + Sync,
{
    fn resolve_async<'a>(
        &'a self,
        info: &'a Self::TypeInfo,
        _: Option<&'a [Selection<S>]>,
        executor: &'a Executor<Self::Context, S>,
    ) -> crate::BoxFuture<'a, ExecutionResult<S>> {
        let f = async move {
            let value = match self {
                Some(obj) => executor.resolve_into_value_async(info, obj).await,
                None => Value::null(),
            };
            Ok(value)
        };
        Box::pin(f)
    }
}

impl<S, T> FromInputValue<S> for Option<T>
where
    T: FromInputValue<S>,
    S: ScalarValue,
{
    fn from_input_value(v: &InputValue<S>) -> Option<Self> {
        match v {
            &InputValue::Null => Some(None),
            v => v.convert().map(Some),
        }
    }
}

impl<S, T> ToInputValue<S> for Option<T>
where
    T: ToInputValue<S>,
    S: ScalarValue,
{
    fn to_input_value(&self) -> InputValue<S> {
        match *self {
            Some(ref v) => v.to_input_value(),
            None => InputValue::null(),
        }
    }
}

impl<S, T> GraphQLType<S> for Vec<T>
where
    T: GraphQLType<S>,
    S: ScalarValue,
{
    fn name(_: &Self::TypeInfo) -> Option<&'static str> {
        None
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        registry.build_list_type::<T>(info, None).into_meta()
    }
}

impl<S, T> GraphQLValue<S> for Vec<T>
where
    T: GraphQLValue<S>,
    S: ScalarValue,
{
    type Context = T::Context;
    type TypeInfo = T::TypeInfo;

    fn type_name(&self, _: &Self::TypeInfo) -> Option<&'static str> {
        None
    }

    fn resolve(
        &self,
        info: &Self::TypeInfo,
        _: Option<&[Selection<S>]>,
        executor: &Executor<Self::Context, S>,
    ) -> ExecutionResult<S> {
        resolve_into_list(executor, info, self.iter())
    }
}

impl<S, T> GraphQLValueAsync<S> for Vec<T>
where
    T: GraphQLValueAsync<S>,
    T::TypeInfo: Sync,
    T::Context: Sync,
    S: ScalarValue + Send + Sync,
{
    fn resolve_async<'a>(
        &'a self,
        info: &'a Self::TypeInfo,
        _: Option<&'a [Selection<S>]>,
        executor: &'a Executor<Self::Context, S>,
    ) -> crate::BoxFuture<'a, ExecutionResult<S>> {
        let f = resolve_into_list_async(executor, info, self.iter());
        Box::pin(f)
    }
}

impl<T, S> FromInputValue<S> for Vec<T>
where
    T: FromInputValue<S>,
    S: ScalarValue,
{
    fn from_input_value(v: &InputValue<S>) -> Option<Self> {
        match *v {
            InputValue::List(ref ls) => {
                let v: Vec<_> = ls.iter().filter_map(|i| i.item.convert()).collect();
                (v.len() == ls.len()).then(|| v)
            }
            ref other => other.convert().map(|e| vec![e]),
        }
    }
}

impl<T, S> ToInputValue<S> for Vec<T>
where
    T: ToInputValue<S>,
    S: ScalarValue,
{
    fn to_input_value(&self) -> InputValue<S> {
        InputValue::list(self.iter().map(T::to_input_value).collect())
    }
}

impl<S, T> GraphQLType<S> for [T]
where
    S: ScalarValue,
    T: GraphQLType<S>,
{
    fn name(_: &Self::TypeInfo) -> Option<&'static str> {
        None
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        registry.build_list_type::<T>(info, None).into_meta()
    }
}

impl<S, T> GraphQLValue<S> for [T]
where
    S: ScalarValue,
    T: GraphQLValue<S>,
{
    type Context = T::Context;
    type TypeInfo = T::TypeInfo;

    fn type_name(&self, _: &Self::TypeInfo) -> Option<&'static str> {
        None
    }

    fn resolve(
        &self,
        info: &Self::TypeInfo,
        _: Option<&[Selection<S>]>,
        executor: &Executor<Self::Context, S>,
    ) -> ExecutionResult<S> {
        resolve_into_list(executor, info, self.iter())
    }
}

impl<S, T> GraphQLValueAsync<S> for [T]
where
    T: GraphQLValueAsync<S>,
    T::TypeInfo: Sync,
    T::Context: Sync,
    S: ScalarValue + Send + Sync,
{
    fn resolve_async<'a>(
        &'a self,
        info: &'a Self::TypeInfo,
        _: Option<&'a [Selection<S>]>,
        executor: &'a Executor<Self::Context, S>,
    ) -> crate::BoxFuture<'a, ExecutionResult<S>> {
        let f = resolve_into_list_async(executor, info, self.iter());
        Box::pin(f)
    }
}

impl<'a, T, S> ToInputValue<S> for &'a [T]
where
    T: ToInputValue<S>,
    S: ScalarValue,
{
    fn to_input_value(&self) -> InputValue<S> {
        InputValue::list(self.iter().map(T::to_input_value).collect())
    }
}

impl<S, T, const N: usize> GraphQLType<S> for [T; N]
where
    S: ScalarValue,
    T: GraphQLType<S>,
{
    fn name(_: &Self::TypeInfo) -> Option<&'static str> {
        None
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        registry.build_list_type::<T>(info, Some(N)).into_meta()
    }
}

impl<S, T, const N: usize> GraphQLValue<S> for [T; N]
where
    S: ScalarValue,
    T: GraphQLValue<S>,
{
    type Context = T::Context;
    type TypeInfo = T::TypeInfo;

    fn type_name(&self, _: &Self::TypeInfo) -> Option<&'static str> {
        None
    }

    fn resolve(
        &self,
        info: &Self::TypeInfo,
        _: Option<&[Selection<S>]>,
        executor: &Executor<Self::Context, S>,
    ) -> ExecutionResult<S> {
        resolve_into_list(executor, info, self.iter())
    }
}

impl<S, T, const N: usize> GraphQLValueAsync<S> for [T; N]
where
    T: GraphQLValueAsync<S>,
    T::TypeInfo: Sync,
    T::Context: Sync,
    S: ScalarValue + Send + Sync,
{
    fn resolve_async<'a>(
        &'a self,
        info: &'a Self::TypeInfo,
        _: Option<&'a [Selection<S>]>,
        executor: &'a Executor<Self::Context, S>,
    ) -> crate::BoxFuture<'a, ExecutionResult<S>> {
        let f = resolve_into_list_async(executor, info, self.iter());
        Box::pin(f)
    }
}

impl<T, S, const N: usize> FromInputValue<S> for [T; N]
where
    T: FromInputValue<S>,
    S: ScalarValue,
{
    fn from_input_value(v: &InputValue<S>) -> Option<Self> {
        struct PartiallyInitializedArray<T, const N: usize> {
            arr: [MaybeUninit<T>; N],
            init_len: usize,
            no_drop: bool,
        }

        impl<T, const N: usize> Drop for PartiallyInitializedArray<T, N> {
            fn drop(&mut self) {
                if self.no_drop {
                    return;
                }
                // Dropping a `MaybeUninit` does nothing, thus we need to drop
                // the initialized elements manually, otherwise we may introduce
                // a memory/resource leak if `T: Drop`.
                for elem in &mut self.arr[0..self.init_len] {
                    // SAFETY: This is safe, because `self.init_len` represents
                    //         exactly the number of initialized elements.
                    unsafe {
                        ptr::drop_in_place(elem.as_mut_ptr());
                    }
                }
            }
        }

        match *v {
            InputValue::List(ref ls) => {
                // SAFETY: The reason we're using a wrapper struct implementing
                //         `Drop` here is to be panic safe:
                //         `T: FromInputValue<S>` implementation is not
                //         controlled by us, so calling `i.item.convert()` below
                //         may cause a panic when our array is initialized only
                //         partially. In such situation we need to drop already
                //         initialized values to avoid possible memory/resource
                //         leaks if `T: Drop`.
                let mut out = PartiallyInitializedArray::<T, N> {
                    // SAFETY: The `.assume_init()` here is safe, because the
                    //         type we are claiming to have initialized here is
                    //         a bunch of `MaybeUninit`s, which do not require
                    //         any initialization.
                    arr: unsafe { MaybeUninit::uninit().assume_init() },
                    init_len: 0,
                    no_drop: false,
                };

                let mut items = ls.iter().filter_map(|i| i.item.convert());
                for elem in &mut out.arr[..] {
                    if let Some(i) = items.next() {
                        *elem = MaybeUninit::new(i);
                        out.init_len += 1;
                    } else {
                        // There is not enough `items` to fill the array.
                        return None;
                    }
                }
                if items.next().is_some() {
                    // There is too much `items` to fit into the array.
                    return None;
                }

                // Do not drop collected `items`, because we're going to return
                // them.
                out.no_drop = true;

                // TODO: Use `mem::transmute` instead of `mem::transmute_copy`
                //       below, once it's allowed for const generics:
                //       https://github.com/rust-lang/rust/issues/61956
                // SAFETY: `mem::transmute_copy` is safe here, because we have
                //         exactly `N` initialized `items`.
                //         Also, despite `mem::transmute_copy` copies the value,
                //         we won't have a double-free when `T: Drop` here,
                //         because original array elements are `MaybeUninit`, so
                //         do nothing on `Drop`.
                Some(unsafe { mem::transmute_copy::<_, Self>(&out.arr) })
            }
            ref other => {
                other.convert().and_then(|e: T| {
                    // TODO: Use `mem::transmute` instead of
                    //       `mem::transmute_copy` below, once it's allowed for
                    //       const generics:
                    //       https://github.com/rust-lang/rust/issues/61956
                    if N == 1 {
                        // SAFETY: `mem::transmute_copy` is safe here, because
                        //         we check `N` to be `1`.
                        //         Also, despite `mem::transmute_copy` copies
                        //         the value, we won't have a double-free when
                        //         `T: Drop` here, because original `e: T` value
                        //         is wrapped into `mem::ManuallyDrop`, so does
                        //         nothing on `Drop`.
                        Some(unsafe {
                            mem::transmute_copy::<_, Self>(&[mem::ManuallyDrop::new(e)])
                        })
                    } else {
                        None
                    }
                })
            }
        }
    }
}

impl<T, S, const N: usize> ToInputValue<S> for [T; N]
where
    T: ToInputValue<S>,
    S: ScalarValue,
{
    fn to_input_value(&self) -> InputValue<S> {
        InputValue::list(self.iter().map(T::to_input_value).collect())
    }
}

fn resolve_into_list<'t, S, T, I>(
    executor: &Executor<T::Context, S>,
    info: &T::TypeInfo,
    iter: I,
) -> ExecutionResult<S>
where
    S: ScalarValue,
    I: Iterator<Item = &'t T> + ExactSizeIterator,
    T: GraphQLValue<S> + ?Sized + 't,
{
    let stop_on_null = executor
        .current_type()
        .list_contents()
        .expect("Current type is not a list type")
        .is_non_null();
    let mut result = Vec::with_capacity(iter.len());

    for o in iter {
        let val = executor.resolve(info, o)?;
        if stop_on_null && val.is_null() {
            return Ok(val);
        } else {
            result.push(val)
        }
    }

    Ok(Value::list(result))
}

async fn resolve_into_list_async<'a, 't, S, T, I>(
    executor: &'a Executor<'a, 'a, T::Context, S>,
    info: &'a T::TypeInfo,
    items: I,
) -> ExecutionResult<S>
where
    I: Iterator<Item = &'t T> + ExactSizeIterator,
    T: GraphQLValueAsync<S> + ?Sized + 't,
    T::TypeInfo: Sync,
    T::Context: Sync,
    S: ScalarValue + Send + Sync,
{
    use futures::stream::{FuturesOrdered, StreamExt as _};

    let stop_on_null = executor
        .current_type()
        .list_contents()
        .expect("Current type is not a list type")
        .is_non_null();

    let mut futures = items
        .map(|it| async move { executor.resolve_into_value_async(info, it).await })
        .collect::<FuturesOrdered<_>>();

    let mut values = Vec::with_capacity(futures.len());
    while let Some(value) = futures.next().await {
        if stop_on_null && value.is_null() {
            return Ok(value);
        }
        values.push(value);
    }

    Ok(Value::list(values))
}
