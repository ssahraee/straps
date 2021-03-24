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

//! Multivariate Polynomials in GF(2)

use itertools::Itertools;
use std::cmp;
use std::fmt;
use std::ops;

pub(crate) type BitSet = bit_set::BitSet<u64>;

/// Multivariate Monomial in GF(2). Ordered first on number of variables, second on variable order.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Monomial(BitSet);

impl fmt::Debug for Monomial {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut vars = self.variables::<u32>();
        match vars.next() {
            Some(v) => {
                write!(f, "x{}", v)?;
                for v in vars {
                    write!(f, "*x{}", v)?;
                }
            }
            None => write!(f, "1")?,
        }
        return Ok(());
    }
}

impl Ord for Monomial {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.0.len().cmp(&other.0.len()) {
            cmp::Ordering::Equal => {
                for i in (0..cmp::max(self.0.capacity(), other.0.capacity())).rev() {
                    let c1 = self.0.contains(i);
                    let c2 = other.0.contains(i);
                    match (c1, c2) {
                        (true, false) => {
                            return cmp::Ordering::Greater;
                        }
                        (false, true) => {
                            return cmp::Ordering::Less;
                        }
                        _ => {}
                    }
                }
                return cmp::Ordering::Equal;
            }
            ne => ne,
        }
    }
}

impl PartialOrd for Monomial {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub(crate) trait PolyIdx: Copy {
    fn to_u32(self) -> u32;
    fn from_u32(x: u32) -> Self;
}

impl PolyIdx for u32 {
    fn to_u32(self) -> u32 {
        self
    }
    fn from_u32(x: u32) -> Self {
        x
    }
}
impl PolyIdx for usize {
    fn to_u32(self) -> u32 {
        self as u32
    }
    fn from_u32(x: u32) -> Self {
        x as usize
    }
}
impl PolyIdx for i32 {
    fn to_u32(self) -> u32 {
        self as u32
    }
    fn from_u32(x: u32) -> Self {
        x as i32
    }
}

/// Multivariate Polynomial in GF(2). Terms are ordered in decreasing order.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Polynomial(Vec<Monomial>);

impl fmt::Debug for Polynomial {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut terms = self.terms();
        match terms.next() {
            Some(t) => {
                write!(f, "{:?}", t)?;
                for t in terms {
                    write!(f, " + {:?}", t)?;
                }
            }
            None => write!(f, "0")?,
        }
        return Ok(());
    }
}

impl Monomial {
    fn divides(&self, div: &Self) -> bool {
        self.0.is_subset(&div.0)
    }
    pub(crate) fn one() -> Self {
        Self(BitSet::default())
    }
    pub(crate) fn variables<'a, T: PolyIdx>(&'a self) -> impl Iterator<Item = T> + 'a {
        self.0.iter().map(|x| T::from_u32(x as u32))
    }
    pub(crate) fn variable_set(&self) -> &BitSet {
        &self.0
    }
    pub(crate) fn from_var<T: PolyIdx>(var: T) -> Self {
        let mut res = Self::one();
        res.0.insert(var.to_u32() as usize);
        return res;
    }
    pub(crate) fn degree(&self) -> u32 {
        self.0.len() as u32
    }
}

impl Polynomial {
    #[cfg(test)]
    pub(crate) fn zero() -> Self {
        Self(Vec::new())
    }
    pub(crate) fn one() -> Self {
        Self::monomial(Monomial::one())
    }
    pub(crate) fn not(self) -> Self {
        self + Self::one()
    }
    pub(crate) fn monomial(mon: Monomial) -> Self {
        Self(vec![mon])
    }
    pub(crate) fn from_var<T: PolyIdx>(var: T) -> Self {
        Self::monomial(Monomial::from_var(var))
    }
    fn first_order_monomials(&self) -> impl Iterator<Item = &Monomial> {
        self.terms().filter(|mon| mon.degree() == 1)
    }
    pub(crate) fn primitive_monomials(&self) -> Vec<Monomial> {
        let pt = self.product_terms();
        self.first_order_monomials()
            .filter(|mon| !mon.divides(&pt))
            .cloned()
            .collect::<Vec<_>>()
    }
    fn product_terms(&self) -> Monomial {
        let mut res = Monomial::one();
        for mon in self.terms() {
            if mon.degree() > 1 {
                res *= mon;
            }
        }
        return res;
    }
    pub(crate) fn variables<T: PolyIdx>(&self) -> impl IntoIterator<Item = T> {
        return self
            .variables_set()
            .iter()
            .map(|x| T::from_u32(x as u32))
            .collect::<Vec<_>>();
    }
    pub(crate) fn variables_set(&self) -> BitSet {
        let mut res = Monomial::one();
        for mon in self.terms() {
            res *= mon;
        }
        return res.0;
    }
    pub(crate) fn terms(&self) -> impl Iterator<Item = &Monomial> + Clone {
        self.0.iter()
    }
    pub(crate) fn from_mon_vec(mut mons: Vec<Monomial>) -> Self {
        mons.sort_unstable_by(|x, y| x.cmp(y).reverse());
        return Polynomial(remove_pairs(mons.into_iter()).collect());
    }
}

impl<'a, 'b> ops::Add<&'a Polynomial> for &'b Polynomial {
    type Output = Polynomial;
    fn add(self, other: &'a Polynomial) -> Self::Output {
        //debug!("{:?} + {:?}", self, other);
        Polynomial(
            self.terms()
                .merge_join_by(other.terms(), |x, y| x.cmp(y).reverse())
                .filter_map(|either| match either {
                    itertools::EitherOrBoth::Left(x) | itertools::EitherOrBoth::Right(x) => Some(x),
                    itertools::EitherOrBoth::Both(_, _) => None,
                })
                .cloned()
                .collect(),
        )
        //debug!("= {:?}", res);
    }
}
impl<'a> ops::Add<&'a Polynomial> for Polynomial {
    type Output = Polynomial;
    fn add(self, other: &'a Polynomial) -> Self::Output {
        &self + other
    }
}
impl<'a> ops::Add<Polynomial> for &'a Polynomial {
    type Output = Polynomial;
    fn add(self, other: Polynomial) -> Self::Output {
        other.add(self)
    }
}
impl ops::Add<Polynomial> for Polynomial {
    type Output = Polynomial;
    fn add(self, other: Polynomial) -> Self::Output {
        self.add(&other)
    }
}
impl<'a> ops::AddAssign<&'a Polynomial> for Polynomial {
    fn add_assign(&mut self, other: &'a Polynomial) {
        *self = &*self + other;
    }
}
impl ops::AddAssign<Polynomial> for Polynomial {
    fn add_assign(&mut self, other: Polynomial) {
        *self = &*self + other;
    }
}
impl ops::AddAssign<Monomial> for Polynomial {
    fn add_assign(&mut self, other: Monomial) {
        match self.0.binary_search_by(|x| x.cmp(&other).reverse()) {
            Ok(i) => {
                self.0.remove(i);
            }
            Err(i) => {
                self.0.insert(i, other);
            }
        }
    }
}
impl<'a> ops::AddAssign<&'a Monomial> for Polynomial {
    fn add_assign(&mut self, other: &'a Monomial) {
        match self.0.binary_search_by(|x| x.cmp(other).reverse()) {
            Ok(i) => {
                self.0.remove(i);
            }
            Err(i) => {
                self.0.insert(i, other.clone());
            }
        }
    }
}
impl ops::Add<Monomial> for Polynomial {
    type Output = Polynomial;
    fn add(mut self, other: Monomial) -> Polynomial {
        self += other;
        return self;
    }
}
impl<'a> ops::Add<&'a Monomial> for Polynomial {
    type Output = Polynomial;
    fn add(mut self, other: &'a Monomial) -> Polynomial {
        self += other;
        return self;
    }
}

impl<'a, 'b> ops::Mul<&'a Monomial> for &'b Monomial {
    type Output = Monomial;
    fn mul(self, other: &'a Monomial) -> Self::Output {
        let mut res = self.clone();
        res *= other;
        return res;
    }
}
impl<'a> ops::Mul<&'a Monomial> for Monomial {
    type Output = Monomial;
    fn mul(mut self, other: &'a Monomial) -> Self::Output {
        self *= other;
        return self;
    }
}
impl<'a> ops::MulAssign<&'a Monomial> for Monomial {
    fn mul_assign(&mut self, other: &'a Monomial) {
        self.0.union_with(&other.0);
    }
}
impl ops::MulAssign<Monomial> for Monomial {
    fn mul_assign(&mut self, other: Monomial) {
        self.mul_assign(&other);
    }
}

impl<'a> ops::Mul<Monomial> for &'a Monomial {
    type Output = Monomial;
    fn mul(self, other: Monomial) -> Self::Output {
        other.mul(self)
    }
}
impl ops::Mul<Monomial> for Monomial {
    type Output = Monomial;
    fn mul(self, other: Monomial) -> Self::Output {
        self.mul(&other)
    }
}

impl<'a, 'b> ops::Mul<&'a Polynomial> for &'b Polynomial {
    type Output = Polynomial;
    fn mul(self, other: &'a Polynomial) -> Self::Output {
        //debug!("{:?} * {:?}", self, other);
        let sum: Vec<Monomial> = self
            .terms()
            .cartesian_product(other.terms())
            .map(|(mon1, mon2)| mon1 * mon2)
            .collect();
        //debug!("sum {:?}", sum);
        return Polynomial::from_mon_vec(sum);
        //debug!("= {:?}", res);
    }
}

impl<'a> ops::Mul<&'a Polynomial> for Polynomial {
    type Output = Polynomial;
    fn mul(self, other: &'a Polynomial) -> Self::Output {
        (&self).mul(other)
    }
}

impl<'a> ops::Mul<Polynomial> for &'a Polynomial {
    type Output = Polynomial;
    fn mul(self, other: Polynomial) -> Self::Output {
        self.mul(&other)
    }
}
impl ops::Mul<Polynomial> for Polynomial {
    type Output = Polynomial;
    fn mul(self, other: Polynomial) -> Self::Output {
        self.mul(&other)
    }
}

fn remove_pairs<I>(it: I) -> impl Iterator<Item = I::Item>
where
    I: Iterator,
    I::Item: Eq,
{
    it.map(|x| Some(x))
        .coalesce(|x, y| if x == y { Ok(None) } else { Err((x, y)) })
        .filter_map(|x| x)
}

#[allow(dead_code)]
pub(crate) fn poly_list_vars<'a>(polys: impl Iterator<Item = &'a Polynomial>) -> BitSet {
    let mut res = BitSet::default();
    for mon in polys.map(|p| p.terms()).flatten() {
        res.union_with(mon.variable_set());
    }
    return res;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anf() {
        let one = Polynomial::one();
        let v1 = Polynomial::from_var(1);
        let v2 = Polynomial::from_var(2);
        let v3 = Polynomial::from_var(3);
        let v4 = Polynomial::from_var(4);
        assert_ne!(one, Polynomial::zero());
        assert_eq!(&one, &one);
        assert_eq!(&one + &one, Polynomial::zero());
        assert_eq!(&v1, &v1);
        assert_eq!(&v1 + &v2, &v2 + &v1);
        assert_eq!(&v3 * (&v1 + &v2), (&v2 + &v1) * &v3);
        assert_eq!(&v1 * (&v1 + &v2), (&v2 + &v1) * &v1);
        let v1_v1v2 = &v4 * &v1 + &v1 * (&v4 + &v1 + &v2);
        assert_eq!(v1_v1v2, (&v2 + &v1) * &v1);
        assert_ne!(&v3 * (&v1 + &v2), (&v2 + &v1) * &v1);
        //assert!(v1_v1v2.contains_primitive_monomial(1));
        //assert!((&v1_v1v2 + &one + &v3).contains_primitive_monomial(1));
        //assert!((&v1_v1v2 + &one + &v3).contains_primitive_monomial(3));
        //assert!(!v1_v1v2.contains_primitive_monomial(2));
        assert_eq!(
            v1_v1v2.first_order_monomials().collect::<Vec<_>>(),
            vec![&Monomial::from_var(1)]
        );
        assert_eq!(
            (&v1_v1v2 + &one + &v3).primitive_monomials(),
            vec![Monomial::from_var(3)]
        );
        assert_eq!(
            (&v1_v1v2 + &v4).primitive_monomials(),
            vec![Monomial::from_var(4)]
        );
        assert_eq!(
            v1_v1v2.product_terms(),
            Monomial::from_var(1) * Monomial::from_var(2)
        );
        assert_eq!(
            (&one + &v3) * (&v1 * &v2 + &v3),
            &v1 * &v2 + &v1 * &v2 * &v3
        );
    }
}
