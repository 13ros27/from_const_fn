use from_const_fn::from_const_fn;

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

#[test]
fn check_correct_generation() {
    let correct: [u8; 50] = core::array::from_fn(|n| n as u8 * 2);
    assert_eq!(correct, MULTIPLES_OF_2_FN);
    assert_eq!(correct, MULTIPLES_OF_2);
    assert_eq!(correct, MULTIPLES_OF_2_BLOCK);
    assert_eq!(correct, MULTIPLES_OF_2_TYPE);
    assert_eq!(correct, MULTIPLES_OF_2_TYPES);
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
