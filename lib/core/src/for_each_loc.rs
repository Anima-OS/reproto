//! Helper trait to iterate over containers of locations
//!
//! This asserts that any errors raised contains location information.

use as_loc::AsLoc;
use std::result;
use {Loc, WithSpan};

/// Helper trait to iterate over a collection of loc items.
pub trait ForEachLoc {
    type Item;

    fn for_each_loc<F, E: WithSpan>(self, callback: F) -> result::Result<(), E>
    where
        F: FnMut(Self::Item) -> result::Result<(), E>;
}

impl<T, I> ForEachLoc for I
where
    I: IntoIterator<Item = T>,
    T: AsLoc,
{
    type Item = T::Output;

    fn for_each_loc<F, E: WithSpan>(self, mut callback: F) -> result::Result<(), E>
    where
        F: FnMut(Self::Item) -> result::Result<(), E>,
    {
        for item in self {
            let (value, span) = Loc::take_pair(item.as_loc());
            callback(value).map_err(|e| e.with_span(span))?;
        }

        Ok(())
    }
}
