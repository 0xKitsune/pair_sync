use std::{
    cmp::Ordering,
    ops::{BitAnd, Div, Shl, Shr, ShrAssign},
    str::FromStr,
    sync::Arc,
};

use ethers::{
    providers::Middleware,
    types::{Log, H160, U256},
};

use crate::{
    dex::{self, DexVariant},
    error::CFMMError,
};

//TODO: FIXME: handle errors gracefully and convert u256 fromstr to const values
pub fn div_uu(x: U256, y: U256) -> u128 {
    if !y.is_zero() {
        let mut answer = U256::zero();

        if x <= U256::from_str("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap() {
            answer = x.shl(64).div(y);
        } else {
            let mut msb = U256::from(192);
            let mut xc = x.shr(192);

            if xc >= U256::from_str("0x100000000").unwrap() {
                xc.shr_assign(32);
                msb += U256::from(32);
            }

            if xc >= U256::from(0x10000) {
                xc.shr_assign(16);
                msb += U256::from(16);
            }

            if xc >= U256::from(0x100) {
                xc.shr_assign(8);
                msb += U256::from(8);
            }

            if xc >= U256::from(0x10) {
                xc.shr_assign(4);
                msb += U256::from(4);
            }

            if xc >= U256::from(0x4) {
                xc.shr_assign(2);
                msb += U256::from(2);
            }

            if xc >= U256::from(0x2) {
                msb += U256::from(1);
            }

            answer = (x.shl(U256::from(255) - msb))
                / (((y - U256::one()) >> (msb - U256::from(191))) + 1);
        }

        if answer > U256::from_str("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap() {
            //TODO: handle this error
            panic!("overflow in divuu")
        }

        let hi = answer * (y.shr(128));
        let mut lo =
            answer * y.bitand(U256::from_str("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap());

        let mut xh = x.shr(192);
        let mut xl = x.shl(64);

        if xl < lo {
            xh -= U256::one();
        }

        xl = xl.overflowing_sub(lo).0;
        lo = hi.shl(128);

        if xl < lo {
            xh -= U256::one();
        }

        xl = xl.overflowing_sub(lo).0;

        if xh != hi.shr(128) {
            //TODO: handle this error
            panic!("assert(xh == hi >> 128);")
        }

        answer += xl / y;

        if answer > U256::from_str("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap() {
            //TODO: handle error
            panic!("overflow in divuu last");
        }

        answer.as_u128()
    } else {
        panic!("bad")
    }
}