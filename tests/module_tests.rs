#![allow(dead_code)]

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
