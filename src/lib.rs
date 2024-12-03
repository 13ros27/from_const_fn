#![no_std]
use core::mem::ManuallyDrop;

#[macro_export]
macro_rules! from_const_fn {
    ($cb:expr) => {{
        /// # Safety
        /// `$cb` must return the same type as the passed function `_`
        const unsafe fn from_const_fn<T, const N: usize>(
            _: ::core::mem::ManuallyDrop<impl FnMut(usize) -> T>, // <-- Note 1
        ) -> [T; N] {
            let mut array = [const { ::core::mem::MaybeUninit::<T>::uninit() }; N];
            let mut i = 0;
            while i < N {
                // SAFETY: `$cb(i)` returns `T` as guaranteed by caller
                array[i] = ::core::mem::MaybeUninit::new(unsafe {
                    $crate::transmute_const($cb(i)) // <-- Note 2
                });
                i += 1;
            }
            // SAFETY: We initialised each `MaybeUninit` in the loop
            //  so we can `assume_init`
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
