//! Zero-copy math on buffers.
//!
//! Are you tired of highly optimized mathematics libraries? Does the hum
//! of a well-utilized arithmatic logic unit fill you with existential dred?
//! Do you find 16-bit integers opulent and excessive?  Boy oh boy, do I have a
//! module for you!
//! 
//! ## Example
//!
//! ```
//! # extern crate ethrpc;
//! # use ethrpc::util::bufmath;
//! # fn main() {
//! 
//! let nine_thousand = [0x23,0x28]; 
//!
//! let fourty_two = [0x00,0x2a];
//! 
//! // All ops are assigning, so we initialize `sum` with
//! // the value of the left-hand operand.
//! let mut sum = nine_thousand.clone();
//!
//! let overflow = bufmath::add(&mut sum, &fourty_two); 
//!
//! assert_eq!(sum,[0x23,0x52]);
//!
//! assert_eq!(overflow,false);
//!
//! // For the untrusting among us:
//! assert!(9000 == 0x2328 && 42 == 0x002a && 9000 + 42 == 0x2352);
//!
//! # }
//! ```


/// Overflowing addition.
///
/// Performs overflowing add-assign between two byte-arrays as if they were
/// big-endian unsigned integers.  Returns `true` if overflow occurred.
///
/// ## Panics
///
/// This function panics if given buffers of differing lengths.
///
#[inline]
pub fn add(lhs: &mut [u8], rhs: &[u8]) -> bool {
    assert_eq!(lhs.len(),rhs.len(),"lhs and rhs buffers have different lengths");
    let mut carry = false;
    for (l,r) in lhs.iter_mut().rev().zip(rhs.iter().rev().cloned()) {
        let (n,o1) = l.overflowing_add(r);
        let (n,o2) = n.overflowing_add(if carry { 1 } else { 0 });
        *l = n;
        carry = o1 || o2;
    }
    carry
}


/// Overflowing subtraction.
///
/// Performs overflowing sub-assign between two byte-arrays as if they were
/// big-endian unsigned integers.  Returns `true` if overflow occurred.
///
/// ## Panics
///
/// This function panics if given buffers of differing lengths.
///
#[inline]
pub fn sub(lhs: &mut [u8], rhs: &[u8]) -> bool {
    assert_eq!(lhs.len(),rhs.len(),"lhs and rhs buffers have different lengths");
    let mut carry = false;
    for (l,r) in lhs.iter_mut().rev().zip(rhs.iter().rev().cloned()) {
        let (n,o1) = l.overflowing_sub(r);
        let (n,o2) = n.overflowing_sub(if carry { 1 } else { 0 });
        *l = n;
        carry = o1 || o2;
    }
    carry
}

#[cfg(test)]
mod test {
    use util::bufmath;
    use rand::{self,Rng};
    use std::mem;


    fn into_bytes(num: u64) -> [u8;8] {
        unsafe { mem::transmute(num.to_be()) }
    }

    fn from_bytes(buf: [u8;8]) -> u64 {
        unsafe { u64::from_be(mem::transmute(buf)) }
    }


    #[test]
    fn conversions() {
        let mut rng = rand::thread_rng();
        for _ in 0..1024 {
            let num = rng.gen::<u64>();
            let buf = into_bytes(num);
            let got = from_bytes(buf);
            assert_eq!(num,got);
        }
    }


    #[test]
    fn large_addition() {
        for a in (0u64..=u64::max_value()).rev().take(512) {
            for b in (0u64..=u64::max_value()).rev().take(512) {
                test_add(a,b);
            }
        }
    }


    #[test]
    fn large_subtraction() {
        for a in (0u64..=u64::max_value()).rev().take(512) {
            for b in (0u64..=u64::max_value()).rev().take(512) {
                test_sub(a,b);
            }
        }
    }

    #[test]
    fn small_addition() {
        for a in 0u64..512u64 {
            for b in 0u64..512u64 {
                test_add(a,b);
            }
        }
    }


    #[test]
    fn small_subtraction() {
        for a in 0u64..512u64 {
            for b in 0u64..512u64 {
                test_sub(a,b);
            }
        }
    }


    #[test]
    fn fuzz_addition() {
        let mut rng = rand::thread_rng();
        for _ in 0..2048 {
            test_add(rng.gen(),rng.gen());
        }
    }


    #[test]
    fn fuzz_subtraction() {
        let mut rng = rand::thread_rng();
        for _ in 0..2048 {
            test_sub(rng.gen(),rng.gen());
        }
    }


    #[inline]
    fn test_add(num_a: u64, num_b: u64) { 
        let buf_a = into_bytes(num_a);
        let buf_b = into_bytes(num_b);
        let (num_sum,num_overflow) = num_a.overflowing_add(num_b);
        let (buf_sum,buf_overflow) = {
            let mut sum = buf_a.clone();
            let overflow = bufmath::add(&mut sum,&buf_b);
            (sum,overflow)
        };
        assert_eq!(num_sum,from_bytes(buf_sum));
        assert_eq!(num_overflow,buf_overflow);
    }


    #[inline]
    fn test_sub(num_a: u64, num_b: u64) {
        let buf_a = into_bytes(num_a);
        let buf_b = into_bytes(num_b);
        let (num_sub,num_overflow) = num_a.overflowing_sub(num_b);
        let (buf_sub,buf_overflow) = {
            let mut sub = buf_a.clone();
            let overflow = bufmath::sub(&mut sub,&buf_b);
            (sub,overflow)
        };
        assert_eq!(num_sub,from_bytes(buf_sub));
        assert_eq!(num_overflow,buf_overflow);
    }
}
