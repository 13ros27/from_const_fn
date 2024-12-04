#![no_std]

/// Like [`array::from_fn`](core::array::from_fn), creates an array of type `[T; N]`,
/// where each element `T` is the returned value from `cb` using that element's index.
///
/// Can be passed functions or closures (except closures borrowing from their environment)
/// taking a single argument of type `usize` and returning values of type `T`.
///
/// Unlike [`array::from_fn`](core::array::from_fn) this also works in `const`,
/// although closures shouldn't start with `const`.
///
/// # Examples
/// ```
/// # use from_const_fn::from_const_fn;
/// const ARRAY: [usize; 5] = from_const_fn!(|i| i);
/// // Indexes are     0  1  2  3  4
/// assert_eq!(ARRAY, [0, 1, 2, 3, 4]);
///
/// const ARRAY_2: [usize; 8] = from_const_fn!(|i| i * 2);
/// // Indexes are      0  1  2  3  4  5   6   7
/// assert_eq!(ARRAY_2, [0, 2, 4, 6, 8, 10, 12, 14]);
///
/// const BOOL_ARRAY: [bool; 5] = from_const_fn!(|i| i % 2 == 0);
/// // Indexes are          0     1      2     3      4
/// assert_eq!(BOOL_ARRAY, [true, false, true, false, true]);
/// ```
#[macro_export]
macro_rules! from_const_fn {
    ($($cb:tt)*) => {{
        $crate::convert_function!($($cb)*);

        /// # Safety
        /// `$cb` must return the same type as the passed function `_`
        const unsafe fn from_const_fn<T: core::fmt::Debug, const N: usize>(
            _: ::core::mem::ManuallyDrop<impl FnMut(usize) -> T>,
        ) -> [T; N] {
            let mut array = [const { ::core::mem::MaybeUninit::<T>::uninit() }; N];

            #[cfg(feature = "drop_guard")]
            {
                let mut guard = $crate::imp::Guard::new(&mut array);
                while guard.get_index() < N {
                    // SAFETY: `$cb(i)` returns `T` as guaranteed by caller
                    let val = unsafe { callback(guard.get_index()) };
                    guard.array[guard.get_index()] = ::core::mem::MaybeUninit::new(val);
                    guard.increment();
                }
                ::core::mem::forget(guard);
            }
            #[cfg(not(feature = "drop_guard"))]
            {
                let mut i = 0;
                while i < N {
                    // SAFETY: `$cb(i)` returns `T` as guaranteed by caller
                    let val = unsafe { callback(i) };
                    array[i] = ::core::mem::MaybeUninit::new(val);
                    i += 1;
                }
            }

            // SAFETY: i == N so the whole array is initialised
            unsafe { $crate::imp::transmute_const(array) }
        }

        // SAFETY: `$cb` is the passed function so it returns the same type.
        unsafe { from_const_fn(::core::mem::ManuallyDrop::new($($cb)*)) }
    }};
}

#[doc(hidden)]
pub mod imp {
    use core::mem::ManuallyDrop;

    #[doc(hidden)]
    #[macro_export]
    macro_rules! convert_function {
        (|$var:ident $(: $_:ident)?| $(-> $__:ident)? $body:expr) => {
            /// # Safety
            /// `$body` must return `T`
            const unsafe fn callback<T>($var: usize) -> T {
                // By placing `$body` in a separate expression we prevent running `unsafe`
                //  code without a visible `unsafe` block
                let body = $body;
                // SAFETY: Guaranteed by caller
                unsafe { $crate::imp::transmute_const(body) }
            }
        };
        ($cb:expr) => {
            /// # Safety
            /// `$cb` must return `T`
            const unsafe fn callback<T>(i: usize) -> T {
                // SAFETY: Guaranteed by caller
                unsafe { $crate::imp::transmute_const($cb(i)) }
            }
        };
    }

    /// Converts `src` into the type `Dst`, checking they are the same size but in
    ///  a less strict way than `mem::transmute`
    /// # Safety
    /// See `mem::transmute`
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

    #[cfg(feature = "drop_guard")]
    pub use drop_guard::Guard;
    #[cfg(feature = "drop_guard")]
    mod drop_guard {
        use core::{mem::MaybeUninit, ptr};

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
                let slice = unsafe {
                    ptr::from_mut(self.array.get_unchecked_mut(..self.index)) as *mut [T]
                };
                // SAFETY:
                //  - `slice` is a pointer formed from a mutable slice so is valid, aligned, nonnull and unique
                //  - The values held in `slice` were generated safely so must uphold their invariants
                unsafe { ptr::drop_in_place(slice) }
            }
        }
    }
}
