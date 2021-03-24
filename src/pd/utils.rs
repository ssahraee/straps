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

use libc::{c_double, c_int};

extern "C" {
    fn ibeta_inv(
        a: c_double,
        b: c_double,
        p: c_double,
        res: *mut c_double,
        py: *mut c_double,
    ) -> c_int;
    fn ibetac_inv(
        a: c_double,
        b: c_double,
        q: c_double,
        res: *mut c_double,
        py: *mut c_double,
    ) -> c_int;
}
fn boost_ibeta_inv(a: f64, b: f64, p: f64) -> Result<(f64, f64), ()> {
    let mut py = 0.0f64;
    let mut res = 0.0f64;
    let ok = unsafe { ibeta_inv(a, b, p, &mut res, &mut py) };
    if ok == 0 {
        return Ok((res, py));
    } else {
        return Err(());
    }
}
fn boost_ibetac_inv(a: f64, b: f64, q: f64) -> Result<(f64, f64), ()> {
    let mut py = 0.0f64;
    let mut res = 0.0f64;
    let ok = unsafe { ibetac_inv(a, b, q, &mut res, &mut py) };
    if ok == 0 {
        return Ok((res, py));
    } else {
        return Err(());
    }
}

/// If k ~ Binom(n, p), compute p' such that Pr[p'>=p] = 1-proba.
/// That is, p' is an upper bound to p with confidence level 1-proba.
/// The result satisfies I_{p'}(k+1,n-k) = 1-proba, where I is the incomplete beta function.
/// Therefore, p' is the result of the inverse complementary beta function ibetac_inv(k+1, n-k,
/// proba)
/// Ref for the formula:
/// Scholz, Fritz. "Confidence bounds and intervals for parameters relating to the binomial,
/// negative binomial, Poisson and hypergeometric distributions with applications to rare events.",
/// 2008.
///
/// Algorithm:
/// Let F(x) be such that CDF_Binom(n, F(x), x) = proba
/// where CDF_bionm(k, n, p) = Pr[x <= k] when x ~ Binom(n, p).
/// Return p' = F(k).
/// Let t be such that CDF_Binom(n, p, t) = proba, we have
/// Pr[k <= t] = proba.
/// Since F is a monotonically increasing function, and since p = F(t), we have p'
/// <= p iff k <= t.
/// Therefore, Pr[p' <= p] = Pr[k <= t] = proba.
/// Computing the inverse CDF: we know that CDF_binom(k, n, p) = I_{1-p}(n-k,k+1).
/// However, for k=n, and proba != 1, there is no p such that CDF_binom(k, n, p) = proba.
/// In case k=n, the only meaningul bound we can return is p'=1.
/// Inverse beta function
pub(crate) fn binom_param_ub(n: u64, k: u64, proba: f64) -> f64 {
    if k == n {
        1.0
    } else {
        boost_ibetac_inv((k + 1) as f64, (n - k) as f64, proba)
            .unwrap()
            .0
    }
}

/// If k ~ Binom(n, p), compute p' such that Pr[p<=p'] = proba.
/// See binom_param_ub for algorithm.
pub(crate) fn binom_param_lb(n: u64, k: u64, proba: f64) -> f64 {
    if k == 0 {
        0.0
    } else {
        let (res, _) = boost_ibeta_inv(k as f64, (n - k + 1) as f64, proba).unwrap();
        assert!(res <= (k as f64) / (n as f64));
        res
    }
}

pub(crate) fn iter_set_bits(mut i: usize) -> impl Iterator<Item = usize> {
    let mut tot = 0;
    std::iter::from_fn(move || {
        if i == 0 {
            None
        } else {
            let nb = i.trailing_zeros() as usize;
            i >>= nb + 1;
            tot += nb + 1;
            Some(tot - 1)
        }
    })
}

#[test]
fn test_iter_set_bits() {
    assert_eq!(iter_set_bits(0b0).collect::<Vec<_>>(), vec![]);
    assert_eq!(iter_set_bits(0b1).collect::<Vec<_>>(), vec![0]);
    assert_eq!(iter_set_bits(0b11).collect::<Vec<_>>(), vec![0, 1]);
    assert_eq!(iter_set_bits(0b111).collect::<Vec<_>>(), vec![0, 1, 2]);
    assert_eq!(iter_set_bits(0b1111).collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    assert_eq!(iter_set_bits(0b101).collect::<Vec<_>>(), vec![0, 2]);
    assert_eq!(iter_set_bits(0b10001).collect::<Vec<_>>(), vec![0, 4]);
    assert_eq!(iter_set_bits(0b10101).collect::<Vec<_>>(), vec![0, 2, 4]);
    assert_eq!(
        iter_set_bits(0b100010001).collect::<Vec<_>>(),
        vec![0, 4, 8]
    );
    assert_eq!(iter_set_bits(0b1000100).collect::<Vec<_>>(), vec![2, 6]);
}
