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
