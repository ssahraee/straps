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

use std::iter::{FromIterator, IntoIterator};
use std::ops::Mul;
pub(crate) type VarIdx = usize;
#[derive(Debug, Clone, Eq, PartialEq, Default, Hash)]
pub(crate) struct BitVarSet(pub(crate) bit_set::BitSet<u64>);

impl std::ops::Deref for BitVarSet {
    type Target = bit_set::BitSet<u64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for BitVarSet {
    fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
        &mut self.0
    }
}

impl Ord for BitVarSet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_ref().cmp(other.get_ref())
    }
}
impl PartialOrd for BitVarSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromIterator<VarIdx> for BitVarSet {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = VarIdx>,
    {
        Self(iter.into_iter().collect())
    }
}
impl Mul<&BitVarSet> for BitVarSet {
    type Output = Self;
    fn mul(mut self, other: &BitVarSet) -> Self::Output {
        self.union_with(other);
        self
    }
}
