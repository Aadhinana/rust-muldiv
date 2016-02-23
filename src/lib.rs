#![crate_name = "muldiv"]
#![feature(asm)]

use std::u8;
use std::u16;
use std::u32;

use std::i64;

use std::cmp;

#[cfg(not(target_arch="x86_64"))]
mod u64_muldiv;
#[cfg(not(target_arch="x86_64"))]
use u64_muldiv::u64_scale;

#[cfg(target_arch="x86_64")]
mod u64_muldiv_x86_64;
#[cfg(target_arch="x86_64")]
use u64_muldiv_x86_64::u64_scale;

pub trait MulDiv<RHS = Self> {
    type Output;

    fn mul_div_floor(self, num: RHS, denom: RHS) -> Self::Output;
    fn mul_div_round(self, num: RHS, denom: RHS) -> Self::Output;
    fn mul_div_ceil(self, num: RHS, denom: RHS) -> Self::Output;
}

impl MulDiv for u64 {
    type Output = Option<u64>;

    fn mul_div_floor(self, num: u64, denom: u64) -> Option<u64> {
        assert!(denom != 0);
        u64_scale(self, num, denom, 0)
    }

    fn mul_div_round(self, num: u64, denom: u64) -> Option<u64> {
        assert!(denom != 0);
        u64_scale(self, num, denom, denom >> 1)
    }

    fn mul_div_ceil(self, num: u64, denom: u64) -> Option<u64> {
        assert!(denom != 0);
        u64_scale(self, num, denom, denom - 1)
    }
}

#[cfg(test)]
mod muldiv_u64_tests {
    use super::*;
    use std::u32;
    use std::u64;

    extern crate num;
    extern crate rand;

    use self::num::bigint::ToBigUint;
    use self::num::integer::Integer;
    use self::rand::thread_rng;
    use self::rand::Rng;

    #[test]
    fn scale_floor() {
        assert_eq!(Some(1), 1u64.mul_div_floor(1, 1));

        assert_eq!(Some(100), 10u64.mul_div_floor(10, 1));
        assert_eq!(Some(50), 10u64.mul_div_floor(10, 2));

        assert_eq!(Some(0), 0u64.mul_div_floor(10, 2));
        assert_eq!(Some(0), 0u64.mul_div_floor(0, 2));

        assert_eq!(Some((u32::MAX as u64) * 5), (u32::MAX as u64).mul_div_floor(5, 1));
        assert_eq!(Some((u32::MAX as u64) * 5), (u32::MAX as u64).mul_div_floor(10, 2));

        assert_eq!(Some((u32::MAX as u64) / 5), (u32::MAX as u64).mul_div_floor(1, 5));
        assert_eq!(Some((u32::MAX as u64) / 5), (u32::MAX as u64).mul_div_floor(2, 10));

        // not quite overflow
        assert_eq!(Some(0xe666666666666664), (u64::MAX - 1).mul_div_floor(9, 10));
        assert_eq!(Some(0xfffffffefffffffd), (u64::MAX - 1).mul_div_floor(u32::MAX as u64 - 1, u32::MAX as u64));
        assert_eq!(Some(0xfffffffeffffff9a), (u64::MAX - 100).mul_div_floor(u32::MAX as u64 - 1, u32::MAX as u64));

        assert_eq!(Some(0xfffffffffffffffd), (u64::MAX - 1).mul_div_floor(u64::MAX - 1, u64::MAX));
        assert_eq!(Some(0xffffffffffffff9a), (u64::MAX - 100).mul_div_floor(u64::MAX - 1, u64::MAX));

        // overflow
        assert_eq!(None, (u64::MAX - 1).mul_div_floor(10, 1));
        assert_eq!(None, (u64::MAX - 1).mul_div_floor(u64::MAX, 1));
    }

    #[test]
    fn scale_floor_rng() {
        let mut rng = thread_rng();

        // 64 bit scaling
        for _ in 0..10000 {
            let val: u32 = rng.gen();
            let num: u32 = rng.gen();
            let den: u32 = rng.gen();

            if den == 0 { continue; }

            let res = (val as u64).mul_div_floor(num as u64, den as u64);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (expected, _) = (val_big * num_big).div_rem(&den_big);

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: u64 = rng.gen();
            let num: u32 = rng.gen();
            let den: u32 = rng.gen();

            if den == 0 { continue; }

            let res = val.mul_div_floor(num as u64, den as u64);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (expected, _) = (val_big * num_big).div_rem(&den_big);

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: u32 = rng.gen();
            let num: u64 = rng.gen();
            let den: u32 = rng.gen();

            if den == 0 { continue; }

            let res = (val as u64).mul_div_floor(num, den as u64);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (expected, _) = (val_big * num_big).div_rem(&den_big);

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 128 bit scaling
        for _ in 0..10000 {
            let val: u64 = rng.gen();
            let num: u64 = rng.gen();
            let den: u64 = rng.gen();

            if den == 0 { continue; }

            let res = val.mul_div_floor(num, den);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (expected, _) = (val_big * num_big).div_rem(&den_big);

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }
    }

    #[test]
    fn scale_round_rng() {
        let mut rng = thread_rng();

        // 64 bit scaling
        for _ in 0..10000 {
            let val: u32 = rng.gen();
            let num: u32 = rng.gen();
            let den: u32 = rng.gen();

            if den == 0 { continue; }

            let res = (val as u64).mul_div_round(num as u64, den as u64);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if expected_rem >= den_big >> 1 { expected = expected + 1.to_biguint().unwrap() }

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: u64 = rng.gen();
            let num: u32 = rng.gen();
            let den: u32 = rng.gen();

            if den == 0 { continue; }

            let res = val.mul_div_round(num as u64, den as u64);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if expected_rem >= den_big >> 1 { expected = expected + 1.to_biguint().unwrap() }

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: u32 = rng.gen();
            let num: u64 = rng.gen();
            let den: u32 = rng.gen();

            if den == 0 { continue; }

            let res = (val as u64).mul_div_round(num, den as u64);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if expected_rem >= den_big >> 1 { expected = expected + 1.to_biguint().unwrap() }

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 128 bit scaling
        for _ in 0..10000 {
            let val: u64 = rng.gen();
            let num: u64 = rng.gen();
            let den: u64 = rng.gen();

            if den == 0 { continue; }

            let res = val.mul_div_round(num, den);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if expected_rem >= den_big >> 1 { expected = expected + 1.to_biguint().unwrap() }

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }
    }

    #[test]
    fn scale_ceil_rng() {
        let mut rng = thread_rng();

        // 64 bit scaling
        for _ in 0..10000 {
            let val: u32 = rng.gen();
            let num: u32 = rng.gen();
            let den: u32 = rng.gen();

            if den == 0 { continue; }

            let res = (val as u64).mul_div_ceil(num as u64, den as u64);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if expected_rem != 0.to_biguint().unwrap() { expected = expected + 1.to_biguint().unwrap() }

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: u64 = rng.gen();
            let num: u32 = rng.gen();
            let den: u32 = rng.gen();

            if den == 0 { continue; }

            let res = val.mul_div_ceil(num as u64, den as u64);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if expected_rem != 0.to_biguint().unwrap() { expected = expected + 1.to_biguint().unwrap() }

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: u32 = rng.gen();
            let num: u64 = rng.gen();
            let den: u32 = rng.gen();

            if den == 0 { continue; }

            let res = (val as u64).mul_div_ceil(num, den as u64);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if expected_rem != 0.to_biguint().unwrap() { expected = expected + 1.to_biguint().unwrap() }

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 128 bit scaling
        for _ in 0..10000 {
            let val: u64 = rng.gen();
            let num: u64 = rng.gen();
            let den: u64 = rng.gen();

            if den == 0 { continue; }

            let res = val.mul_div_ceil(num, den);

            let val_big = val.to_biguint().unwrap();
            let num_big = num.to_biguint().unwrap();
            let den_big = den.to_biguint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if expected_rem != 0.to_biguint().unwrap() { expected = expected + 1.to_biguint().unwrap() }

            if expected > u64::MAX.to_biguint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_biguint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }
    }

}

fn abs_u64(x: i64) -> u64 {
    if x != i64::MIN { x.abs() as u64 }
    else { 0x8000000000000000u64 }
}

impl MulDiv for i64 {
    type Output = Option<i64>;

    fn mul_div_floor(self, num: i64, denom: i64) -> Option<i64> {
        assert!(denom != 0);

        let sgn = self.signum() * num.signum() * denom.signum();

        let self_u = abs_u64(self);
        let num_u = abs_u64(num);
        let denom_u = abs_u64(denom);

        if sgn < 0 {
            self_u.mul_div_ceil(num_u, denom_u)
        } else {
            self_u.mul_div_floor(num_u, denom_u)
        }.and_then(|r| if r < i64::MAX as u64 { Some(sgn * (r as i64)) }
                          else if sgn < 0 && r == 0x8000000000000000u64 { Some(i64::MIN) }
                          else { None }
                  )
    }

    fn mul_div_round(self, num: i64, denom: i64) -> Option<i64> {
        assert!(denom != 0);

        let sgn = self.signum() * num.signum() * denom.signum();

        let self_u = abs_u64(self);
        let num_u = abs_u64(num);
        let denom_u = abs_u64(denom);

        if sgn < 0 {
            u64_scale(self_u, num_u, denom_u, cmp::max(denom_u >> 1, 1) - 1)
        } else {
            self_u.mul_div_round(num_u, denom_u)
        }.and_then(|r| if r < i64::MAX as u64 { Some(sgn * (r as i64)) }
                       else if sgn < 0 && r == 0x8000000000000000u64 { Some(i64::MIN) }
                       else { None }
                  )
    }

    fn mul_div_ceil(self, num: i64, denom: i64) -> Option<i64> {
        assert!(denom != 0);

        let sgn = self.signum() * num.signum() * denom.signum();

        let self_u = abs_u64(self);
        let num_u = abs_u64(num);
        let denom_u = abs_u64(denom);

        if sgn < 0 {
            self_u.mul_div_floor(num_u, denom_u)
        } else {
            self_u.mul_div_ceil(num_u, denom_u)
        }.and_then(|r| if r < i64::MAX as u64 { Some(sgn * (r as i64)) }
                       else if sgn < 0 && r == 0x8000000000000000u64 { Some(i64::MIN) }
                       else { None }
                  )
    }
}

#[cfg(test)]
mod muldiv_i64_tests {
    use super::*;
    use std::i64;

    extern crate num;
    extern crate rand;

    use self::num::integer::Integer;
    use self::num::traits::Signed;
    use self::num::bigint::ToBigInt;
    use self::rand::thread_rng;
    use self::rand::Rng;

    #[test]
    fn scale_floor_rng() {
        let mut rng = thread_rng();

        // 64 bit scaling
        for _ in 0..10000 {
            let val: i32 = rng.gen();
            let num: i32 = rng.gen();
            let den: i32 = rng.gen();
            let sgn = val.signum() * num.signum() * den.signum();

            if den == 0 { continue; }

            let res = (val as i64).mul_div_floor(num as i64, den as i64);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn < 0 && expected_rem.abs() != 0.to_bigint().unwrap() { expected = expected - 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: i64 = rng.gen();
            let num: i32 = rng.gen();
            let den: i32 = rng.gen();
            let sgn = (val.signum() as i32) * num.signum() * den.signum();

            if den == 0 { continue; }

            let res = val.mul_div_floor(num as i64, den as i64);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn < 0 && expected_rem.abs() != 0.to_bigint().unwrap() { expected = expected - 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: i32 = rng.gen();
            let num: i64 = rng.gen();
            let den: i32 = rng.gen();
            let sgn = val.signum() * (num.signum() as i32) * den.signum();

            if den == 0 { continue; }

            let res = (val as i64).mul_div_floor(num, den as i64);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn < 0 && expected_rem.abs() != 0.to_bigint().unwrap() { expected = expected - 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 128 bit scaling
        for _ in 0..10000 {
            let val: i64 = rng.gen();
            let num: i64 = rng.gen();
            let den: i64 = rng.gen();
            let sgn = val.signum() * num.signum() * den.signum();

            if den == 0 { continue; }

            let res = val.mul_div_floor(num, den);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn < 0 && expected_rem.abs() != 0.to_bigint().unwrap() { expected = expected - 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }
    }

    #[test]
    fn scale_round_rng() {
        let mut rng = thread_rng();

        // 64 bit scaling
        for _ in 0..10000 {
            let val: i32 = rng.gen();
            let num: i32 = rng.gen();
            let den: i32 = rng.gen();
            let sgn = val.signum() * num.signum() * den.signum();

            if den == 0 { continue; }

            let res = (val as i64).mul_div_round(num as i64, den as i64);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn < 0 && expected_rem.abs() > den_big.abs() >> 1 { expected = expected - 1.to_bigint().unwrap() }
            else if sgn > 0 && expected_rem.abs() >= den_big.abs() >> 1 { expected = expected + 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: i64 = rng.gen();
            let num: i32 = rng.gen();
            let den: i32 = rng.gen();
            let sgn = (val.signum() as i32) * num.signum() * den.signum();

            if den == 0 { continue; }

            let res = val.mul_div_round(num as i64, den as i64);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn < 0 && expected_rem.abs() > den_big.abs() >> 1 { expected = expected - 1.to_bigint().unwrap() }
            else if sgn > 0 && expected_rem.abs() >= den_big.abs() >> 1 { expected = expected + 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: i32 = rng.gen();
            let num: i64 = rng.gen();
            let den: i32 = rng.gen();
            let sgn = val.signum() * (num.signum() as i32) * den.signum();

            if den == 0 { continue; }

            let res = (val as i64).mul_div_round(num, den as i64);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn < 0 && expected_rem.abs() > den_big.abs() >> 1 { expected = expected - 1.to_bigint().unwrap() }
            else if sgn > 0 && expected_rem.abs() >= den_big.abs() >> 1 { expected = expected + 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 128 bit scaling
        for _ in 0..10000 {
            let val: i64 = rng.gen();
            let num: i64 = rng.gen();
            let den: i64 = rng.gen();
            let sgn = val.signum() * num.signum() * den.signum();

            if den == 0 { continue; }

            let res = val.mul_div_round(num, den);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn < 0 && expected_rem.abs() > den_big.abs() >> 1 { expected = expected - 1.to_bigint().unwrap() }
            else if sgn > 0 && expected_rem.abs() >= den_big.abs() >> 1 { expected = expected + 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }
    }

    #[test]
    fn scale_ceil_rng() {
        let mut rng = thread_rng();

        // 64 bit scaling
        for _ in 0..10000 {
            let val: i32 = rng.gen();
            let num: i32 = rng.gen();
            let den: i32 = rng.gen();
            let sgn = val.signum() * num.signum() * den.signum();

            if den == 0 { continue; }

            let res = (val as i64).mul_div_ceil(num as i64, den as i64);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn > 0 && expected_rem.abs() != 0.to_bigint().unwrap() { expected = expected + 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: i64 = rng.gen();
            let num: i32 = rng.gen();
            let den: i32 = rng.gen();
            let sgn = (val.signum() as i32) * num.signum() * den.signum();

            if den == 0 { continue; }

            let res = val.mul_div_ceil(num as i64, den as i64);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn > 0 && expected_rem.abs() != 0.to_bigint().unwrap() { expected = expected + 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 96 bit scaling
        for _ in 0..10000 {
            let val: i32 = rng.gen();
            let num: i64 = rng.gen();
            let den: i32 = rng.gen();
            let sgn = val.signum() * (num.signum() as i32) * den.signum();

            if den == 0 { continue; }

            let res = (val as i64).mul_div_ceil(num, den as i64);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn > 0 && expected_rem.abs() != 0.to_bigint().unwrap() { expected = expected + 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }

        // 128 bit scaling
        for _ in 0..10000 {
            let val: i64 = rng.gen();
            let num: i64 = rng.gen();
            let den: i64 = rng.gen();
            let sgn = val.signum() * num.signum() * den.signum();

            if den == 0 { continue; }

            let res = val.mul_div_ceil(num, den);

            let val_big = val.to_bigint().unwrap();
            let num_big = num.to_bigint().unwrap();
            let den_big = den.to_bigint().unwrap();
            let (mut expected, expected_rem) = (val_big * num_big).div_rem(&den_big);

            if sgn > 0 && expected_rem.abs() != 0.to_bigint().unwrap() { expected = expected + 1.to_bigint().unwrap() }

            if expected > i64::MAX.to_bigint().unwrap() || expected < i64::MIN.to_bigint().unwrap() {
                assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
            } else {
                assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                assert!(res.unwrap().to_bigint().unwrap() == expected, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
            }
        }
    }
}

macro_rules! mul_div_impl_unsigned {
    ($t:ident, $u:ident) => (

    impl MulDiv for $t {
        type Output = Option<$t>;

        fn mul_div_floor(self, num: $t, denom: $t) -> Option<$t> {
            assert!(denom != 0);
            let r = ((self as $u) * (num as $u)) / (denom as $u);
            if r > $t::MAX as $u { None } else { Some(r as $t) }
        }

        fn mul_div_round(self, num: $t, denom: $t) -> Option<$t> {
            assert!(denom != 0);
            let r = ((self as $u) * (num as $u) + ((denom >> 1) as $u)) / (denom as $u);
            if r > $t::MAX as $u { None } else { Some(r as $t) }
        }

        fn mul_div_ceil(self, num: $t, denom: $t) -> Option<$t> {
            assert!(denom != 0);
            let r = ((self as $u) * (num as $u) + ((denom - 1) as $u)) / (denom as $u);
            if r > $t::MAX as $u { None } else { Some(r as $t) }
        }
    }
    )
}

macro_rules! mul_div_impl_unsigned_tests {
    ($t:ident, $u:ident) => (
        use super::*;

        extern crate num;
        extern crate rand;

        use self::num::integer::Integer;
        use self::rand::thread_rng;
        use self::rand::Rng;
        use std::$t;

        #[test]
        fn scale_floor_rng() {
            let mut rng = thread_rng();

            for _ in 0..10000 {
                let val: $t = rng.gen();
                let num: $t = rng.gen();
                let den: $t = rng.gen();

                if den == 0 { continue; }

                let res = val.mul_div_floor(num, den);

                let (expected, _) = ((val as $u) * (num as $u)).div_rem(&(den as $u));

                if expected > $t::MAX as $u {
                    assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
                } else {
                    assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                    assert!(res.unwrap() == expected as $t, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
                }
            }
        }

        // FIXME: This fails because of wrong rounding in the test
        #[test]
        #[ignore]
        fn scale_round_rng() {
            let mut rng = thread_rng();

            for _ in 0..10000 {
                let val: $t = rng.gen();
                let num: $t = rng.gen();
                let den: $t = rng.gen();

                if den == 0 { continue; }

                let res = val.mul_div_round(num, den);

                let (mut expected, expected_rem) = ((val as $u) * (num as $u)).div_rem(&(den as $u));

                if expected_rem > (den as $u) >> 1 { expected = expected + 1 }

                if expected > $t::MAX as $u {
                    assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
                } else {
                    assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                    assert!(res.unwrap() == expected as $t, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
                }
            }
        }

        #[test]
        fn scale_ceil_rng() {
            let mut rng = thread_rng();

            for _ in 0..10000 {
                let val: $t = rng.gen();
                let num: $t = rng.gen();
                let den: $t = rng.gen();

                if den == 0 { continue; }

                let res = val.mul_div_ceil(num, den);

                let (mut expected, expected_rem) = ((val as $u) * (num as $u)).div_rem(&(den as $u));

                if expected_rem != 0 { expected = expected + 1 }

                if expected > $t::MAX as $u {
                    assert!(res.is_none(), format!("{} * {} / {}: expected overflow, got {}", val, num, den, res.unwrap()));
                } else {
                    assert!(res.is_some(), format!("{} * {} / {}: expected {} but got overflow", val, num, den, expected));
                    assert!(res.unwrap() == expected as $t, format!("{} * {} / {}: expected {} but got {}", val, num, den, expected, res.unwrap()));
                }
            }
        }

    )
}

mul_div_impl_unsigned!(u32, u64);
mul_div_impl_unsigned!(u16, u32);
mul_div_impl_unsigned!(u8, u16);

// FIXME: https://github.com/rust-lang/rust/issues/12249
#[cfg(test)]
mod muldiv_u32_tests {
    mul_div_impl_unsigned_tests!(u32, u64);
}

#[cfg(test)]
mod muldiv_u16_tests {
    mul_div_impl_unsigned_tests!(u16, u32);
}

#[cfg(test)]
mod muldiv_u8_tests {
    mul_div_impl_unsigned_tests!(u8, u16);
}

// TODO: Add s32/16/8 implementations
