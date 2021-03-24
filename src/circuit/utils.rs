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
