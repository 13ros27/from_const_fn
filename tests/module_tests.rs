use from_const_fn::from_const_fn;

mod alias {
    pub type Alias<T> = T;
}

const fn multiply_by_2(n: usize) -> u8 {
    n as u8 * 2
}

const MULTIPLES_OF_2_FN: [u8; 50] = from_const_fn!(multiply_by_2);
const MULTIPLES_OF_2: [u8; 50] = from_const_fn!(|n| n as u8 * 2);
const MULTIPLES_OF_2_BLOCK: [u8; 50] = from_const_fn!(|n| {
    let n_cast = n as u8;
    n_cast * 2
});

const MULTIPLES_OF_2_TYPE: [u8; 50] = from_const_fn!(|n: usize| n as u8 * 2);
const MULTIPLES_OF_2_TYPES: [u8; 50] = from_const_fn!(|n: usize| -> u8 { n as u8 * 2 });
// Checks that the macro is using `$ty` not `$ident` (thanks u/AlxandrHeintz)
const MULTIPLES_OF_2_TYPES_GEN: [u8; 50] =
    from_const_fn!(|n: alias::Alias<usize>| -> alias::Alias<u8> { n as u8 * 2 });
// Ensure a multi-line body works
const MULTIPLES_OF_2_TYPES_COMPLEX_BODY: [u8; 50] = from_const_fn!(|n| -> u8 {
    let n_cast = n as u8;
    n_cast * 2
});

const WILDCARD_CLOSURE: [u8; 50] = from_const_fn!(|_| 35);
const WILDCARD_CLOSURE_BLOCK: [u8; 50] = from_const_fn!(|_| { 35 });
const WILDCARD_CLOSURE_TYPE: [u8; 50] = from_const_fn!(|_| -> u8 { 35 });

#[test]
fn check_correct_generation() {
    let correct: [u8; 50] = [
        0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46,
        48, 50, 52, 54, 56, 58, 60, 62, 64, 66, 68, 70, 72, 74, 76, 78, 80, 82, 84, 86, 88, 90, 92,
        94, 96, 98,
    ];
    assert_eq!(correct, MULTIPLES_OF_2_FN);
    assert_eq!(correct, MULTIPLES_OF_2);
    assert_eq!(correct, MULTIPLES_OF_2_BLOCK);

    assert_eq!(correct, MULTIPLES_OF_2_TYPE);
    assert_eq!(correct, MULTIPLES_OF_2_TYPES);
    assert_eq!(correct, MULTIPLES_OF_2_TYPES_GEN);
    assert_eq!(correct, MULTIPLES_OF_2_TYPES_COMPLEX_BODY);

    assert_eq!([35; 50], WILDCARD_CLOSURE);
    assert_eq!([35; 50], WILDCARD_CLOSURE_BLOCK);
    assert_eq!([35; 50], WILDCARD_CLOSURE_TYPE);

    #[cfg(feature = "drop_guard")]
    {
        use core::cell::UnsafeCell;
        struct SyncUnsafeCell(UnsafeCell<u8>);
        // SAFETY: Haha nope
        unsafe impl Sync for SyncUnsafeCell {}
        static COUNTER: SyncUnsafeCell = SyncUnsafeCell(UnsafeCell::new(0));
        let wildcard_closure_counting: [u8; 50] = from_const_fn!(|_| {
            let n = COUNTER.0.get();
            unsafe { *n += 2 };
            unsafe { *n - 2 }
        });
        assert_eq!(correct, wildcard_closure_counting);
    }
}

#[cfg(feature = "drop_guard")]
mod drop_tests {
    use super::*;
    use core::sync::atomic::{AtomicU32, Ordering};
    use std::panic::catch_unwind;

    #[test]
    fn drop_check() {
        static DROP_COUNTER: AtomicU32 = AtomicU32::new(0);
        #[derive(Debug)]
        struct Dropped;
        impl Drop for Dropped {
            fn drop(&mut self) {
                DROP_COUNTER.fetch_add(1, Ordering::Relaxed);
            }
        }

        catch_unwind(|| {
            let _generated: [Dropped; 10] = from_const_fn!(|n| {
                if n >= 5 {
                    panic!();
                }
                Dropped
            });
        })
        .ok();
        assert_eq!(DROP_COUNTER.load(Ordering::Relaxed), 5);
    }
}
