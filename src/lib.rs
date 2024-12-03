#![no_std]
use core::mem::{ManuallyDrop, MaybeUninit};

#[macro_export]
macro_rules! from_const_fn {
    ($cb:expr, $n:expr) => {{
        let mut array = [const { ::core::mem::MaybeUninit::uninit() }; $n];
        let mut i = 0;
        while i < $n {
            array[i] = ::core::mem::MaybeUninit::new($cb(i));
            i += 1;
        }
        // SAFETY: We initialised each `MaybeUninit` in the loop
        //  so we can `assume_init`
        unsafe { $crate::array_assume_init(array) }
    }};
}

/// Converts `src` into the type `Dst`, checking they are the same size but in
///  a less strict way than `mem::transmute`
/// # Safety
/// See `mem::transmute`
#[doc(hidden)]
const unsafe fn transmute_const<Src, Dst>(src: Src) -> Dst {
    const fn check_equal<Src, Dst>() {
        assert!(size_of::<Src>() == size_of::<Dst>());
    }
    const { check_equal::<Src, Dst>() };

    // SAFETY:
    //  - We checked `size_of::<Src>() == size_of::<Dst>()` above
    //  - Everything else guaranteed by caller
    unsafe { transmute_unchecked::<Src, Dst>(src) }
}

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

/// # Safety
/// It is up to the caller to guarantee that all elements of the array are
///  in an initialized state.
#[doc(hidden)]
pub const unsafe fn array_assume_init<T, const N: usize>(array: [MaybeUninit<T>; N]) -> [T; N] {
    // SAFETY: Guaranteed by caller
    unsafe { transmute_const(array) }
}
