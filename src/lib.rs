//! This crate provides DangerousOption - a type similar to `!` in Swift language. It's basically
//! an `Option` which panics if dereferenced while containing `None`. This is useful in case one
//! needs to initialize things a little bit later or when accesses are made via functions called
//! from trusted library.
//!
//! While such thing might look like a step back (there's a reason we don't have NULL pointers in
//! Rust), there is still one advantage over classic approach of NULL-pointer exceptions (including
//! manually unwrapping): the cause of the bug is usually not in the place where dereferencing
//! happened but in the place where assignment happened. Since this type has only three, very
//! unique methods for creating invalid value, those can be easily searched for and tracked.
//!
//! It has also intentionally long name to prevent over-use. Also the methods creating dangerous
//! state have longer names then those creating valid state.
//!
//! Note: you should prefer dereferencing the DangerousOption and passing dereferenced value
//! instead of passing the reference to DangerousOption itself. That way you'll decrease chance of
//! making mistake.
//!
//! Finally, it also provides an exception handler which allows customizing panic message, logging,
//! etc. There is a default handler which just panics, but in contexts where there is a more
//! concrete, known cause of invalid operation, overriding the message is encouraged.
//!
//! This crate is `no_std`.

#![no_std]

/// The exception handler defining behavior in case `None` is accessed.
pub trait ExceptionHandler {
    /// Called when dereferencing of `None` is attempted.
    fn bad_deref() -> !;

    /// Called on attempt to take out value from `Some`, if there is `None`.
    fn bad_take() -> !;
}

/// This is the default handler for `None` exceptions.
pub enum DefaultExceptionHandler {}

impl ExceptionHandler for DefaultExceptionHandler {
    fn bad_deref() -> ! {
        panic!("Dereferenced uninitialized DangerousOption")
    }

    fn bad_take() -> ! {
        panic!("Attempt to take value from uninitialized DangerousOption")
    }
}

/// Represents a value that might be uninitialized, but most probably isn't. It provides convenient
/// access to the value via `Deref` while checking whether the value is actually initialized.
///
/// When deref of initialized value is attempted, the ExceptionHandler is called. This will lead to
/// aborting of the task.
#[derive(Debug)]
pub struct DangerousOption<T, H: ExceptionHandler = DefaultExceptionHandler>(Option<T>, core::marker::PhantomData<H>);

impl<T, H: ExceptionHandler> core::ops::Deref for DangerousOption<T, H> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap_or_else(|| H::bad_deref())
    }
}

impl<T, H: ExceptionHandler> core::ops::DerefMut for DangerousOption<T, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap_or_else(|| H::bad_deref())
    }
}

impl<T, H: ExceptionHandler> DangerousOption<T, H> {
    /// Creates valid value.
    pub fn new(val: T) -> Self {
        DangerousOption(Some(val), Default::default())
    }

    /// Creates uninitialized value.
    pub fn new_uninitialized() -> Self {
        DangerousOption(None, Default::default())
    }

    /// Takes out the value, failing if it's not there. After call to this function, the value is
    /// uninitialized.
    pub fn take_unchecked(this: &mut Self) -> T {
        this.0.take().unwrap_or_else(|| H::bad_take())
    }

    /// Tries to take out the value. After call to this function, the value is uninitialized.
    pub fn take_checked(this: &mut Self) -> Option<T> {
        this.0.take()
    }

    /// Non-panicking version of deref, which returns `None`, if value is uninitiaized.
    pub fn try(this: &Self) -> Option<&T> {
        this.0.as_ref()
    }

    /// Non-panicking version of deref_mut, which returns `None`, if value is uninitiaized.
    pub fn try_mut(this: &mut Self) -> Option<&mut T> {
        this.0.as_mut()
    }

    /// Puts the new value in place of old, optionally returning old value.
    pub fn put(this: &mut Self, val: T) -> Option<T> {
        core::mem::replace(&mut this.0, Some(val))
    }
}

impl<T> core::clone::Clone for DangerousOption<T> where T : Clone {
    fn clone(&self) -> Self {
        DangerousOption(self.0.clone(), Default::default())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn success() {
        use ::DangerousOption;

        let mut val: DangerousOption<i32> = DangerousOption::new(42);
        assert_eq!(*val, 42);
        {
            let ref mut val2 = *val;
            assert_eq!(*val2, 42);
            *val2 = 47;
        }

        let val2 = DangerousOption::take_unchecked(&mut val);
        assert_eq!(val2, 47);
        assert!(DangerousOption::try(&val).is_none());
        DangerousOption::put(&mut val, val2);
        assert_eq!(DangerousOption::take_unchecked(&mut val), 47);
        assert!(DangerousOption::try(&val).is_none());
        DangerousOption::put(&mut val, val2);
        assert_eq!(*DangerousOption::try(&val).unwrap(), 47);
        {
            let ref mut val2 = *DangerousOption::try_mut(&mut val).unwrap();
            assert_eq!(*val2, 47);
            *val2 = 42;
        }
        assert_eq!(*val, 42);
    }

    #[test]
    #[should_panic]
    fn panic1() {
        use ::DangerousOption;
        use core::mem::drop;

        let val: DangerousOption<i32> = DangerousOption::new_uninitialized();
        drop(*val);
    }

    #[test]
    #[should_panic]
    fn panic2() {
        use ::DangerousOption;

        let mut val: DangerousOption<i32> = DangerousOption::new_uninitialized();
        DangerousOption::take_unchecked(&mut val);
    }

    #[test]
    #[should_panic]
    fn panic3() {
        use ::DangerousOption;

        let mut val: DangerousOption<i32> = DangerousOption::new_uninitialized();
        let ref mut val2 = *val;
        *val2 = 42;
    }
}
