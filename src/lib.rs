#![cfg_attr(not(feature = "std"), no_std)]
use core::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

#[macro_export]
macro_rules! from_const_fn {
    ($cb:expr) => {{
        /// # Safety
        /// `$cb` must return the same type as the passed function `_`
        const unsafe fn from_const_fn<T, const N: usize>(
            _: ::core::mem::ManuallyDrop<impl FnMut(usize) -> T>, // <-- Note 1
        ) -> [T; N] {
            let mut array = [const { ::core::mem::MaybeUninit::<T>::uninit() }; N];
            let mut guard = $crate::Guard::new(&mut array);
            while guard.get_index() < N {
                let ret = $cb(guard.get_index());
                // SAFETY: `$cb(i)` returns `T` as guaranteed by caller
                let val = unsafe { $crate::transmute_const(ret) };
                guard.array[guard.get_index()] = ::core::mem::MaybeUninit::new(val);
                guard.increment();
            }
            ::core::mem::forget(guard);
            // SAFETY: i == N so the whole array is initialised
            unsafe { $crate::transmute_const(array) }
        }

        // SAFETY: `$cb` is the passed function so it returns the same type.
        unsafe { from_const_fn(::core::mem::ManuallyDrop::new($cb)) }
    }};
}

/// Converts `src` into the type `Dst`, checking they are the same size but in
///  a less strict way than `mem::transmute`
/// # Safety
/// See `mem::transmute`
#[doc(hidden)]
pub const unsafe fn transmute_const<Src, Dst>(src: Src) -> Dst {
    const fn check_equal<Src, Dst>() {
        assert!(size_of::<Src>() == size_of::<Dst>());
    }
    const { check_equal::<Src, Dst>() };

    // SAFETY:
    //  - We checked `size_of::<Src>() == size_of::<Dst>()` above
    //  - Everything else guaranteed by caller
    unsafe { transmute_unchecked::<Src, Dst>(src) }
}

/// Converts `src` into the type `Dst` without checking they're the same size
/// # Safety
///  - The caller must follow all invariants of `mem::transmute`
///  - `size_of::<Src>() == size_of::<Dst>()`
const unsafe fn transmute_unchecked<Src, Dst>(src: Src) -> Dst {
    #[repr(C)]
    union Transmute<Src, Dst> {
        src: ManuallyDrop<Src>,
        dst: ManuallyDrop<Dst>,
    }

    let alchemy = Transmute {
        src: ManuallyDrop::new(src),
    };
    // SAFETY: Guaranteed by caller
    unsafe { ManuallyDrop::into_inner(alchemy.dst) }
}

#[doc(hidden)]
pub struct Guard<'a, T, const N: usize> {
    pub array: &'a mut [MaybeUninit<T>; N],
    index: usize,
}

impl<'a, T, const N: usize> Guard<'a, T, N> {
    pub const fn new(array: &'a mut [MaybeUninit<T>; N]) -> Self {
        Self { array, index: 0 }
    }

    pub const fn get_index(&self) -> usize {
        self.index
    }

    /// # Safety
    ///  - `self.array` must be initialised up to and including the new `self.index`
    ///  - `self.array.len()` must be greater than `self.index`
    pub const unsafe fn increment(&mut self) {
        self.index += 1;
    }
}

impl<T, const N: usize> Drop for Guard<'_, T, N> {
    fn drop(&mut self) {
        // SAFETY: `array` must be initialised up to `index` so we can reinterpret a slice up to there as `[T]`
        let slice =
            unsafe { ptr::from_mut(self.array.get_unchecked_mut(..self.index)) as *mut [T] };
        // SAFETY:
        //  - `slice` is a pointer formed from a mutable slice so is valid, aligned, nonnull and unique
        //  - The values held in `slice` were generated safely so must uphold their invariants
        unsafe { ptr::drop_in_place(slice) }
        #[cfg(feature = "std")]
        eprintln!("Panicked, dropped {} items", self.index);
    }
}
