// STRAPS - Statistical Testing of RAndom Probing Security
// Copyright (C) 2021 UCLouvain
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Consider a probe distribution, which is a vector x of 2^n elements.
//! We consider an index i to be a superset of an index j if (j & ~i) == 0.
//! The cumulative transform y=T(x) of x is such that y[j] = sum_i x[i] for all i that are
//! supersets of j.  The naive implementation of the transform (or its inverse) is quadratic in the
//! size of the distribution.
//!
//! A more efficient (2^n log 2^n) algorithm is based on the following observation:
//! If y=T(x), and (x1,x2)=x, (y1,y2)=y (evenly split the two vectors), then
//! y2=T(x2), and furthermore y1=y2+T(x1).
//!
//! We also consider the inverse transform, and a "positive" variant of it:
//! given y: find a x whose elements are all positive such that T(x) >= y (element-wise), while
//! minimizing the elements of x (minimizing first the elements whose indexes have the largest
//! hamming weight).
//! In a similar way, we consider the min_positive inverse variant that satisfies T(x) <= y, while
//! maximizing the elements of x (maximizing first the elements whose indexes have the largest
//! hamming weight).

use itertools::Itertools;
use num_traits::Zero;
use std::ops::{Add, Sub};

pub trait Group: Add<Self, Output = Self> + Sub<Self, Output = Self> + Zero {}
impl<T> Group for T where T: Add<T, Output = T> + Sub<T, Output = T> + Zero {}

/// Cumulative transform (naive implementation). Do not use, use cum_transform instead.
/// Only for correctness testing and benchamrk.
pub fn cum_transform_naive<T: Group + Copy>(x: &mut [T]) {
    let src = x.to_vec();
    let n = x.len();
    for (i, x) in x.iter_mut().enumerate() {
        *x = (0..n)
            // take all ii that are supersets of i
            .filter(|ii| (i & !ii) == 0)
            .map(|ii| src[ii])
            .fold(<T as Zero>::zero(), |s, x| s + x);
    }
}

/// Cumulative transform (recursive function). Do not use, use cum_transform instead.
/// Only for correctness testing and benchamrk.
pub fn cum_transform_rec<T: Group + Copy>(x: &mut [T]) {
    if x.len() != 1 {
        let half = x.len() / 2;
        let (x, y) = x.split_at_mut(half);
        cum_transform_rec(x);
        cum_transform_rec(y);
        x.iter_mut().zip(y.iter()).for_each(|(x, y)| *x = *x + *y);
    }
}

/// Inverse cumulative transform (recursive function). Do not use, use cum_transform_inv instead.
/// Only for correctness testing and benchamrk.
pub fn cum_transform_inv_rec<T: Group + Copy>(x: &mut [T]) {
    if x.len() != 1 {
        let half = x.len() / 2;
        let (x, y) = x.split_at_mut(half);
        x.iter_mut().zip(y.iter()).for_each(|(x, y)| *x = *x - *y);
        cum_transform_inv_rec(x);
        cum_transform_inv_rec(y);
    }
}

/// Inverse positive cumulative transform (recursive function).
/// Do not use, use cum_transform_inv_positive instead.
/// Only for correctness testing and benchamrk.
pub fn cum_transform_inv_rec_positive<T: Group + Copy + Ord>(x: &mut [T]) {
    if x.len() != 1 {
        let half = x.len() / 2;
        let (x, y) = x.split_at_mut(half);
        cum_transform_inv_rec_positive(y);
        let mut yp = y.to_vec();
        cum_transform_rec(&mut yp);
        x.iter_mut()
            .zip(yp.iter())
            .for_each(|(x, y)| *x = std::cmp::max(T::zero(), *x - *y));
        cum_transform_inv_rec_positive(x);
    }
}

/// Inverse min positive cumulative transform (recursive function).
/// Do not use, use cum_transform_inv_min_positive instead.
/// Only for correctness testing and benchamrk.
pub fn cum_transform_inv_min_rec_positive<T: Group + Copy + Ord>(x: &mut [T]) {
    if x.len() != 1 {
        let half = x.len() / 2;
        let (x, y) = x.split_at_mut(half);
        y.iter_mut()
            .zip(x.iter())
            .for_each(|(y, x)| *y = std::cmp::min(*y, *x));
        cum_transform_inv_min_rec_positive(y);
        let mut yp = y.to_vec();
        cum_transform_rec(&mut yp);
        assert!(x.iter().zip(yp.iter()).all(|(x, y)| *x >= *y));
        x.iter_mut().zip(yp.iter()).for_each(|(x, y)| *x = *x - *y);
        cum_transform_inv_min_rec_positive(x);
    }
}

/// Cumulative transform.
pub fn cum_transform<T: Group + Copy>(x: &mut [T]) {
    let mut half_block_size = 1;
    while half_block_size * 2 <= x.len() {
        x.chunks_exact_mut(half_block_size)
            .tuples()
            .for_each(|(x, y)| x.iter_mut().zip(y.iter()).for_each(|(x, y)| *x = *x + *y));
        half_block_size *= 2;
    }
}

/// Inverse cumulative transform.
pub fn cum_transform_inv<T: Group + Copy>(x: &mut [T]) {
    let mut half_block_size = x.len() / 2;
    while half_block_size >= 1 {
        x.chunks_exact_mut(half_block_size)
            .tuples()
            .for_each(|(x, y)| x.iter_mut().zip(y.iter()).for_each(|(x, y)| *x = *x - *y));
        half_block_size /= 2;
    }
}

/// Inner function fo cum_transform_inv_positive
/// x: vector to be transformed
/// xr: to be written to: the transform of the result
/// Performance note: this is still written as a recursive function, probably could be optimized a
/// bit more (seems to loose a factor of ~3 in benchmarks with size 2^12 compared to
/// cum_transform_inv).
fn cum_transform_inv_positive_inner<T: Group + Copy + Ord>(x: &mut [T], xr: &mut [T]) {
    if x.len() == 1 {
        xr[0] = x[0];
    } else {
        let half = x.len() / 2;
        let (x, y) = x.split_at_mut(half);
        let (xr, yr) = xr.split_at_mut(half);
        cum_transform_inv_positive_inner(y, yr);
        x.iter_mut()
            .zip(yr.iter())
            .for_each(|(x, y)| *x = std::cmp::max(T::zero(), *x - *y));
        cum_transform_inv_positive_inner(x, xr);
        xr.iter_mut()
            .zip(yr.iter())
            .for_each(|(xr, yr)| *xr = *xr + *yr);
    }
}

/// Inverse positive cumulative transform (recursive function).
pub fn cum_transform_inv_positive<T: Group + Copy + Ord>(x: &mut [T]) {
    cum_transform_inv_positive_inner(x, &mut vec![T::zero(); x.len()]);
}

/// Inner function fo cum_transform_inv_min_positive
/// x: vector to be transformed
/// xr: to be written to: the transform of the result
fn cum_transform_inv_min_positive_inner<T: Group + Copy + Ord + std::fmt::Debug>(
    x: &mut [T],
    xr: &mut [T],
) {
    //let mut x_test = x.to_vec();
    if x.len() == 1 {
        xr[0] = x[0];
    } else {
        let half = x.len() / 2;
        let (x, y) = x.split_at_mut(half);
        let (xr, yr) = xr.split_at_mut(half);
        y.iter_mut()
            .zip(x.iter())
            .for_each(|(y, x)| *y = std::cmp::min(*y, *x));
        cum_transform_inv_min_positive_inner(y, yr);
        //assert!(x.iter().zip(yr.iter()).all(|(x, y)| *x >= *y));
        // The max is needed only to compensate for rounding errors.
        x.iter_mut()
            .zip(yr.iter())
            .for_each(|(x, y)| *x = std::cmp::max(T::zero(), *x - *y));
        cum_transform_inv_min_positive_inner(x, xr);
        xr.iter_mut()
            .zip(yr.iter())
            .for_each(|(xr, yr)| *xr = *xr + *yr);
    }
    /*
    let mut xt = x.to_vec();
    cum_transform(&mut xt);
    //assert_eq!(xr, &mut xt);
    let mut x_test2 = x_test.clone();
    let mut xr_test = x_test.clone();
    cum_transform_inv_positive_inner(&mut x_test2, &mut xr_test);
    assert!(
        xr_test.iter().zip(x_test.iter()).all(|(xr, x)| *xr >= *x),
        "x_test: {:?}, xr_test: {:?}, x_test2: {:?}",
        x_test,
        xr_test,
        x_test2
    );
    assert!(xr.iter().zip(x_test.iter()).all(|(xr, x)| *xr <= *x));
    assert!(
        xr.iter().zip(xr_test.iter()).all(|(x, xr)| *x <= *xr),
        "x: {:?}, xr: {:?}, xrt: {:?}",
        x_test,
        xr,
        xr_test
    );
    */
}

/// Inverse minimum positive cumulative transform (recursive function).
pub fn cum_transform_inv_min_positive<T: Group + Copy + Ord + std::fmt::Debug>(x: &mut [T]) {
    cum_transform_inv_min_positive_inner(x, &mut vec![T::zero(); x.len()]);
}

#[test]
fn test_rec_transform_small() {
    let v = vec![0, 1, 2, 3];
    let vt = vec![6, 4, 5, 3];
    let mut w = v.clone();
    cum_transform_rec(&mut w);
    assert_eq!(w, vt);
    let mut w = v.clone();
    cum_transform_naive(&mut w);
    assert_eq!(w, vt);
    let mut w = vt.clone();
    cum_transform_inv_rec(&mut w);
    assert_eq!(v, w);
    let mut w = vt.clone();
    cum_transform_inv_rec_positive(&mut w);
    assert_eq!(v, w);
    let mut w = vt.clone();
    cum_transform_inv_positive(&mut w);
    assert_eq!(v, w);
    let mut wt = vt.clone();
    cum_transform_inv_min_rec_positive(&mut wt);
    assert!(
        wt.iter().zip(v.iter()).all(|(w, v)| *w <= *v),
        "w: {:?}, v: {:?}",
        wt,
        v
    );
    let mut w = vt.clone();
    cum_transform_inv_min_positive(&mut w);
    assert_eq!(wt, w);
    let mut w = v.clone();
    cum_transform(&mut w);
    assert_eq!(w, vt);
    let mut w = vt.clone();
    cum_transform_inv(&mut w);
    assert_eq!(v, w);
}
#[test]
fn test_rec_transform_big() {
    let v = (0..(1 << 10)).collect::<Vec<_>>();
    let mut vt = v.clone();
    cum_transform_rec(&mut vt);
    let mut w = v.clone();
    cum_transform_naive(&mut w);
    assert_eq!(vt, w);
    let mut w = vt.clone();
    cum_transform_inv_rec(&mut w);
    assert_eq!(v, w);
    let mut w = vt.clone();
    cum_transform_inv_rec_positive(&mut w);
    assert_eq!(v, w);
    let mut w = vt.clone();
    cum_transform_inv_positive(&mut w);
    assert_eq!(v, w);
    let mut wt = vt.clone();
    cum_transform_inv_min_rec_positive(&mut wt);
    assert!(wt.iter().zip(v.iter()).all(|(w, v)| *w <= *v));
    let mut w = vt.clone();
    cum_transform_inv_min_positive(&mut w);
    assert_eq!(wt, w);
    let mut w = v.clone();
    cum_transform(&mut w);
    assert_eq!(vt, w);
    let mut w = vt.clone();
    cum_transform_inv(&mut w);
    assert_eq!(v, w);
}
#[test]
fn test_rec_transform_positive() {
    let v = vec![1, -1, -1, 1, 1, -1, -1, 1];
    let vp = vec![0, 0, 0, 1, 0, 0, 0, 1];
    let mut vt = v.clone();
    cum_transform_rec(&mut vt);
    let mut w = v.clone();
    cum_transform(&mut w);
    assert_eq!(vt, w);
    let mut w = v.clone();
    cum_transform_naive(&mut w);
    assert_eq!(vt, w);
    let mut w = vt.clone();
    cum_transform_inv_rec_positive(&mut w);
    assert_eq!(vp, w);
    let mut w = vt.clone();
    cum_transform_inv_positive(&mut w);
    assert_eq!(vp, w);
}
