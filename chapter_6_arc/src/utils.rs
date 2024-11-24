use std::ptr::NonNull;

pub(crate) fn non_null_from<T>(x: T) -> NonNull<T> {
    NonNull::from(Box::leak(Box::new(x)))
}
