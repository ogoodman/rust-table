//! Comparison functions for `i64` and `f64`.
//!
//! Although such comparisons are perfectly well-defined
//! (absent `NaN`) Rust, in common with most languages, does not
//! implement them. The difficulty is that not every 64-bit
//! integer can be represented as a 64-bit floating point
//! number or vice-versa.
//!
//! The solution is to consider a number of cases. Integers of
//! magnitude < 2**53 can be converted losslessly to floating
//! point. If one number has magnitude < 2**53 and the other
//! greater, the result depends only on the sign of the latter.
//! Floating point numbers of magnitude > 2**53 are integral
//! and thus can either be converted losslessly to `i64` or lie
//! outside the range of `i64`.
//!
//! To provide a total ordering we define `NaN` to be smaller
//! than any number.

use std::cmp::Ordering;

/// Returns an `Ordering` between an `i64` and an `f64`.
pub fn cmp_if(n: i64, x: f64) -> Ordering {
    if x.is_nan() {
        // We decide (arbitrarily) that NaN is smaller than any other number.
        return Ordering::Greater;
    }

    let f_resi = 1i64 << 53;
    let f_resf = (53f64).exp2();
    let int_lim = (63f64).exp2();

    if -f_resi < n && n < f_resi {
        // n can be converted losslessly to f64; x is not NaN so unwrap must succeed.
        (n as f64).partial_cmp(&x).unwrap()
    } else if x.abs() < f_resf {
        // abs_n dominates abs_x so result depends only on sign of n.
        if n > 0 {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    } else if x >= int_lim {
        // x bigger than any i64.
        Ordering::Less
    } else if x < -int_lim {
        // x smaller than any i64.
        Ordering::Greater
    } else {
        // abs_x >= f_resf implies that x represents an integer.
        n.cmp(&(x as i64))
    }
}

/// Returns an `Ordering` between an `f64` and an `i64`.
pub fn cmp_fi(x: f64, n: i64) -> Ordering {
    if x.is_nan() {
        // We decide (arbitrarily) that NaN is smaller than any other number.
        return Ordering::Less;
    }

    let f_resi = 1i64 << 53;
    let f_resf = (53f64).exp2();
    let int_lim = (63f64).exp2();

    if -f_resi < n && n < f_resi {
        // n can be converted losslessly to f64; x is not NaN so unwrap must succeed.
        x.partial_cmp(&(n as f64)).unwrap()
    } else if x.abs() < f_resf {
        // abs_n dominates abs_x so result depends only on sign of n.
        if n > 0 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    } else if x >= int_lim {
        // x bigger than any i64.
        Ordering::Greater
    } else if x < -int_lim {
        // x smaller than any i64.
        Ordering::Less
    } else {
        // abs_x >= f_resf implies that x represents an integer.
        (x as i64).cmp(&n)
    }
}

/// Returns an `Ordering` between two `f64`s.
///
/// This treats `NaN` as smaller than any number so that the ordering
/// can be total.

pub fn cmp_ff(x: f64, y: f64) -> Ordering {
    if x.is_nan() {
        if y.is_nan() {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    } else {
        if y.is_nan() {
            Ordering::Greater
        } else {
            x.partial_cmp(&y).unwrap()
        }
    }
}

