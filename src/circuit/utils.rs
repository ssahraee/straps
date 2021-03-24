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

use std::hash::Hash;

struct ItComplement<W, B, T>
where
    W: Iterator<Item = T>,
    B: Iterator<Item = T>,
    T: Ord,
{
    world: W,
    base: std::iter::Peekable<B>,
}
impl<W, B, T> Iterator for ItComplement<W, B, T>
where
    W: Iterator<Item = T>,
    B: Iterator<Item = T>,
    T: Ord,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        loop {
            let next = self.world.next()?;
            loop {
                let sub = self.base.peek();
                if let Some(x) = sub {
                    if x < &next {
                        self.base.next();
                    } else if x == &next {
                        self.base.next();
                        break;
                    } else {
                        return Some(next);
                    }
                } else {
                    return Some(next);
                }
            }
        }
    }
}

pub(crate) fn it_complement<T: Ord>(
    world: impl Iterator<Item = T>,
    base: impl Iterator<Item = T>,
) -> impl Iterator<Item = T> {
    ItComplement {
        world,
        base: base.peekable(),
    }
}

/// Test if there are no duplicates in an iterator
pub(crate) fn is_unique<I>(it: I) -> bool
where
    I: Iterator,
    I::Item: Clone + Eq + Hash,
{
    let mut set: fxhash::FxHashSet<_> = Default::default();
    for x in it {
        if !set.insert(x.clone()) {
            return false;
        }
    }
    return true;
}
