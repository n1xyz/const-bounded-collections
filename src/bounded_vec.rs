use alloc::vec;
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};
use core::slice::{Iter, IterMut};
use thiserror::Error;

/// Non-empty Vec bounded with minimal (L - lower bound) and maximal (U - upper bound) items quantity.
///
/// # Type Parameters
///
/// * `W` - witness type to prove vector ranges and shape it interface accordingly
#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct BoundedVec<T, const L: usize, const U: usize, W = witnesses::NonEmpty<L, U>> {
    inner: Vec<T>,
    _marker: core::marker::PhantomData<W>,
}

/// BoundedVec errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum BoundedVecOutOfBounds {
    /// Items quantity is less than L (lower bound)
    #[error("Lower bound violation: got {got} (expected >= {lower_bound})")]
    LowerBoundError {
        /// L (lower bound)
        lower_bound: usize,
        /// provided value
        got: usize,
    },
    /// Items quantity is more than U (upper bound)
    #[error("Upper bound violation: got {got} (expected <= {upper_bound})")]
    UpperBoundError {
        /// U (upper bound)
        upper_bound: usize,
        /// provided value
        got: usize,
    },
}

/// Module for type witnesses used to prove vector bounds at compile time
pub mod witnesses {
    // NOTE:
    // we can have proves if needed for some cases like 8/16/32/64 upper bound and operating range,
    // and make memory layout more efficient:
    // - decide stackalloc or smallvec or std::vec, depending on range * size_of at compile time
    // - make some values of vec to be not usize, but other numbers

    /// Compile-time proof of valid bounds. Must be consturcted with same bounds to instantiate `BoundedVec`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct NonEmpty<const L: usize, const U: usize>(());

    /// Possibly empty vector with upper bound.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Empty<const U: usize>(());

    /// Type a compile-time proof of valid bounds
    pub const fn non_empty<const L: usize, const U: usize>() -> NonEmpty<L, U> {
        const {
            if L == 0 {
                panic!("L must be greater than 0")
            }
            if L > U {
                panic!("L must be less than or equal to U")
            }

            serde::<U>();
            NonEmpty::<L, U>(())
        }
    }

    const fn serde<const U: usize>() {
        #[cfg(feature = "schema")]
        if U as u128 > u32::MAX as u128 {
            // there is not const safe way to cast usize to u32, nor to other bigger number
            panic!("`schemars` encodes `maxLength` as u32, so `U` must be less than or equal to `u32::MAX`")
        }

        #[cfg(feature = "borsh")]
        if U as u128 > u32::MAX as u128 {
            panic!("`borsh` specifies size of dynamic containers as u32, so `U` must be less than or equal to `u32::MAX`")
        }
    }

    /// Type a compile-time proof for possibly empty vector with upper bound
    pub const fn empty<const U: usize>() -> Empty<U> {
        const {
            serde::<U>();
            Empty::<U>(())
        }
    }
}

impl<T, const U: usize> BoundedVec<T, 0, U, witnesses::Empty<U>> {
    /// Creates new BoundedVec or returns error if items count is out of bounds
    ///
    /// # Parameters
    ///
    /// * `items` - vector of items within bounds
    ///
    /// # Errors
    ///
    /// * `UpperBoundError` - if `items`` len is more than U (upper bound)
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::witnesses;
    /// let data: BoundedVec<_, 0, 8, witnesses::Empty<8>> =
    ///     BoundedVec::<_, 0, 8, witnesses::Empty<8>>::from_vec(vec![1u8, 2]).unwrap();
    /// ```
    pub fn from_vec(items: Vec<T>) -> Result<Self, BoundedVecOutOfBounds> {
        let _ = witnesses::empty::<U>();
        let len = items.len();
        if len > U {
            Err(BoundedVecOutOfBounds::UpperBoundError {
                upper_bound: U,
                got: len,
            })
        } else {
            Ok(BoundedVec {
                inner: items,
                _marker: core::marker::PhantomData,
            })
        }
    }

    /// Returns the first element of the vector, or `None` if it is empty
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::witnesses;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<u8, 0, 8, witnesses::Empty<8>> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.first(), Some(&1u8));
    /// ```
    pub fn first(&self) -> Option<&T> {
        self.inner.first()
    }

    /// Returns `true` if the vector contains no elements
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::witnesses;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<u8, 0, 8, witnesses::Empty<8>> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the last element of the vector, or `None` if it is empty
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::witnesses;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<u8, 0, 8, witnesses::Empty<8>> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.last(), Some(&2u8));
    /// ```
    pub fn last(&self) -> Option<&T> {
        self.inner.last()
    }
}

/// Part which works for all witnesses
impl<T, const L: usize, const U: usize, W> BoundedVec<T, L, U, W> {
    /// Returns a reference to underlying `Vec``
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.as_vec(), &vec![1u8,2]);
    /// ```
    pub fn as_vec(&self) -> &Vec<T> {
        &self.inner
    }

    /// Returns an underlying `Vec``
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.to_vec(), vec![1u8,2]);
    /// ```
    pub fn to_vec(self) -> Vec<T> {
        self.inner
    }

    /// Extracts a slice containing the entire vector.
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.as_slice(), &[1u8,2]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }

    /// Returns a reference for an element at index or `None` if out of bounds
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let elem = *data.get(1).unwrap();
    /// assert_eq!(elem, 2);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    /// Returns an iterator
    pub fn iter(&self) -> Iter<T> {
        self.inner.iter()
    }

    /// Returns an iterator that allows to modify each value
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.inner.iter_mut()
    }
}

impl<T, const L: usize, const U: usize> BoundedVec<T, L, U, witnesses::NonEmpty<L, U>> {
    /// Creates new BoundedVec or returns error if items count is out of bounds
    ///
    /// # Parameters
    ///
    /// * `items` - vector of items within bounds
    ///
    /// # Errors
    ///
    /// * `LowerBoundError` - if `items`` len is less than L (lower bound)
    /// * `UpperBoundError` - if `items`` len is more than U (upper bound)
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::witnesses;
    /// let data: BoundedVec<_, 2, 8, witnesses::NonEmpty<2, 8>> =
    ///     BoundedVec::<_, 2, 8, witnesses::NonEmpty<2, 8>>::from_vec(vec![1u8, 2]).unwrap();
    /// ```
    pub fn from_vec(items: Vec<T>) -> Result<Self, BoundedVecOutOfBounds> {
        let _ = witnesses::non_empty::<L, U>();
        let len = items.len();
        if len < L {
            Err(BoundedVecOutOfBounds::LowerBoundError {
                lower_bound: L,
                got: len,
            })
        } else if len > U {
            Err(BoundedVecOutOfBounds::UpperBoundError {
                upper_bound: U,
                got: len,
            })
        } else {
            Ok(BoundedVec {
                inner: items,
                _marker: core::marker::PhantomData,
            })
        }
    }

    /// Returns the number of elements in the vector
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<u8, 2, 4> = vec![1u8,2].try_into().unwrap();
    /// assert_eq!(data.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the first element of non-empty Vec
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(*data.first(), 1);
    /// ```
    pub fn first(&self) -> &T {
        #[allow(clippy::unwrap_used)]
        self.inner.first().unwrap()
    }

    /// Returns the last element of non-empty Vec
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(*data.last(), 2);
    /// ```
    pub fn last(&self) -> &T {
        #[allow(clippy::unwrap_used)]
        self.inner.last().unwrap()
    }

    /// Create a new `BoundedVec` by consuming `self` and mapping each element.
    ///
    /// This is useful as it keeps the knowledge that the length is >= U, <= L,
    /// even through the old `BoundedVec` is consumed and turned into an iterator.
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let data = data.mapped(|x|x*2);
    /// assert_eq!(data, [2u8,4].into());
    /// ```
    pub fn mapped<F, N>(self, map_fn: F) -> BoundedVec<N, L, U, witnesses::NonEmpty<L, U>>
    where
        F: FnMut(T) -> N,
    {
        BoundedVec {
            inner: self.inner.into_iter().map(map_fn).collect::<Vec<_>>(),
            _marker: core::marker::PhantomData,
        }
    }

    /// Create a new `BoundedVec` by mapping references to the elements of self
    ///
    /// This is useful as it keeps the knowledge that the length is >= U, <= L,
    /// will still hold for new `BoundedVec`
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let data = data.mapped_ref(|x|x*2);
    /// assert_eq!(data, [2u8,4].into());
    /// ```
    pub fn mapped_ref<F, N>(&self, map_fn: F) -> BoundedVec<N, L, U, witnesses::NonEmpty<L, U>>
    where
        F: FnMut(&T) -> N,
    {
        BoundedVec {
            inner: self.inner.iter().map(map_fn).collect::<Vec<_>>(),
            _marker: core::marker::PhantomData,
        }
    }

    /// Create a new `BoundedVec` by consuming `self` and mapping each element
    /// to a `Result`.
    ///
    /// This is useful as it keeps the knowledge that the length is preserved
    /// even through the old `BoundedVec` is consumed and turned into an iterator.
    ///
    /// As this method consumes self, returning an error means that this
    /// vec is dropped. I.e. this method behaves roughly like using a
    /// chain of `into_iter()`, `map`, `collect::<Result<Vec<N>,E>>` and
    /// then converting the `Vec` back to a `Vec1`.
    ///
    ///
    /// # Errors
    ///
    /// Once any call to `map_fn` returns a error that error is directly
    /// returned by this method.
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let data: Result<BoundedVec<u8, 2, 8>, _> = data.try_mapped(|x| Err("failed"));
    /// assert_eq!(data, Err("failed"));
    /// ```
    pub fn try_mapped<F, N, E>(
        self,
        map_fn: F,
    ) -> Result<BoundedVec<N, L, U, witnesses::NonEmpty<L, U>>, E>
    where
        F: FnMut(T) -> Result<N, E>,
    {
        let mut map_fn = map_fn;
        let mut out = Vec::with_capacity(self.len());
        for element in self.inner.into_iter() {
            out.push(map_fn(element)?);
        }
        #[allow(clippy::unwrap_used)]
        Ok(BoundedVec::<N, L, U, witnesses::NonEmpty<L, U>>::from_vec(out).unwrap())
    }

    /// Create a new `BoundedVec` by mapping references of `self` elements
    /// to a `Result`.
    ///
    /// This is useful as it keeps the knowledge that the length is preserved
    /// even through the old `BoundedVec` is consumed and turned into an iterator.
    ///
    /// # Errors
    ///
    /// Once any call to `map_fn` returns a error that error is directly
    /// returned by this method.
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let data: Result<BoundedVec<u8, 2, 8>, _> = data.try_mapped_ref(|x| Err("failed"));
    /// assert_eq!(data, Err("failed"));
    /// ```
    pub fn try_mapped_ref<F, N, E>(
        &self,
        map_fn: F,
    ) -> Result<BoundedVec<N, L, U, witnesses::NonEmpty<L, U>>, E>
    where
        F: FnMut(&T) -> Result<N, E>,
    {
        let mut map_fn = map_fn;
        let mut out = Vec::with_capacity(self.len());
        for element in self.inner.iter() {
            out.push(map_fn(element)?);
        }
        #[allow(clippy::unwrap_used)]
        Ok(BoundedVec::<N, L, U, witnesses::NonEmpty<L, U>>::from_vec(out).unwrap())
    }

    /// Returns the last and all the rest of the elements
    pub fn split_last(&self) -> (&T, &[T]) {
        #[allow(clippy::unwrap_used)]
        self.inner.split_last().unwrap()
    }

    /// Return a new BoundedVec with indices included
    pub fn enumerated(self) -> BoundedVec<(usize, T), L, U, witnesses::NonEmpty<L, U>> {
        #[allow(clippy::unwrap_used)]
        self.inner
            .into_iter()
            .enumerate()
            .collect::<Vec<(usize, T)>>()
            .try_into()
            .unwrap()
    }

    /// Return a Some(BoundedVec) or None if `v` is empty
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::OptBoundedVecToVec;
    ///
    /// let opt_bv_none = BoundedVec::<u8, 2, 8>::opt_empty_vec(vec![]).unwrap();
    /// assert!(opt_bv_none.is_none());
    /// assert_eq!(opt_bv_none.to_vec(), Vec::<u8>::new());
    /// let opt_bv_some = BoundedVec::<u8, 2, 8>::opt_empty_vec(vec![0u8, 2]).unwrap();
    /// assert!(opt_bv_some.is_some());
    /// assert_eq!(opt_bv_some.to_vec(), vec![0u8, 2]);
    /// ```
    pub fn opt_empty_vec(
        v: Vec<T>,
    ) -> Result<Option<BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>>, BoundedVecOutOfBounds> {
        if v.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self::from_vec(v)?))
        }
    }
}

/// A non-empty Vec with no effective upper-bound on its length
pub type NonEmptyVec<T> = BoundedVec<T, 1, { usize::MAX }, witnesses::NonEmpty<1, { usize::MAX }>>;

/// Possibly empty Vec with upper-bound on its length
pub type EmptyBoundedVec<T, const U: usize> = BoundedVec<T, 0, U, witnesses::Empty<U>>;

/// Non-empty Vec with bounded length
pub type NonEmptyBoundedVec<T, const L: usize, const U: usize> =
    BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>;

impl<T, const L: usize, const U: usize> TryFrom<Vec<T>>
    for BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>
{
    type Error = BoundedVecOutOfBounds;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::from_vec(value)
    }
}

impl<T, const U: usize> TryFrom<Vec<T>> for BoundedVec<T, 0, U, witnesses::Empty<U>> {
    type Error = BoundedVecOutOfBounds;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::from_vec(value)
    }
}

// when feature(const_evaluatable_checked) is stable cover all array sizes (L..=U)
impl<T, const L: usize, const U: usize> From<[T; L]>
    for BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>
{
    fn from(arr: [T; L]) -> Self {
        BoundedVec {
            inner: arr.into(),
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T, const L: usize, const U: usize> From<BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>>
    for Vec<T>
{
    fn from(v: BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>) -> Self {
        v.inner
    }
}

impl<T, const L: usize, const U: usize, W> IntoIterator for BoundedVec<T, L, U, W> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T, const L: usize, const U: usize, W> IntoIterator for &'a BoundedVec<T, L, U, W> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, T, const L: usize, const U: usize, W> IntoIterator for &'a mut BoundedVec<T, L, U, W> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl<T, const L: usize, const U: usize, W> AsRef<Vec<T>> for BoundedVec<T, L, U, W> {
    fn as_ref(&self) -> &Vec<T> {
        &self.inner
    }
}

impl<T, const L: usize, const U: usize, W> AsRef<[T]> for BoundedVec<T, L, U, W> {
    fn as_ref(&self) -> &[T] {
        self.inner.as_ref()
    }
}

impl<T, const L: usize, const U: usize, W> AsMut<Vec<T>> for BoundedVec<T, L, U, W> {
    fn as_mut(&mut self) -> &mut Vec<T> {
        self.inner.as_mut()
    }
}

impl<T, const L: usize, const U: usize, W> AsMut<[T]> for BoundedVec<T, L, U, W> {
    fn as_mut(&mut self) -> &mut [T] {
        self.inner.as_mut()
    }
}

/// Option<BoundedVec<T, _, _>> to Vec<T>
pub trait OptBoundedVecToVec<T> {
    /// Option<BoundedVec<T, _, _>> to Vec<T>
    fn to_vec(self) -> Vec<T>;
}

impl<T, const L: usize, const U: usize> OptBoundedVecToVec<T>
    for Option<BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>>
{
    fn to_vec(self) -> Vec<T> {
        self.map(|bv| bv.into()).unwrap_or_default()
    }
}

/// Suports encoding and decoding with [borsh](https://crates.io/crates/borsh), and BorshSchema.
///
/// By default Borsh uses u32 as length prefix for sequences.
/// For bounded we used u8, u16 or u32 depending on the U.
/// Increase or decreaasing U may not always be backward compatible.
#[cfg(feature = "borsh")]
mod borsh_impl {
    use super::*;
    use alloc::collections::btree_map::{BTreeMap, Entry};
    use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

    impl<T: BorshSerialize, const L: usize, const U: usize, W> BorshSerialize
        for BoundedVec<T, L, U, W>
    {
        fn serialize<Writer: borsh::io::Write>(
            &self,
            writer: &mut Writer,
        ) -> borsh::io::Result<()> {
            let len = self.inner.len();
            if U <= usize::from(u8::MAX) {
                #[expect(clippy::expect_used)]
                let len: u8 = len.try_into().expect("proved by design");
                len.serialize(writer)?;
            } else if U <= usize::from(u16::MAX) {
                #[expect(clippy::expect_used)]
                let len: u16 = len.try_into().expect("proved by design");
                len.serialize(writer)?;
            } else {
                #[expect(clippy::expect_used)]
                let len: u32 = len.try_into().expect("proved by design");
                len.serialize(writer)?;
            };

            // adapted from internals of borsh-rs
            let data = self.as_slice();
            if let Some(u8_slice) = T::u8_slice(data) {
                writer.write_all(u8_slice)?;
            } else {
                for item in data {
                    item.serialize(writer)?;
                }
            }
            Ok(())
        }
    }

    impl<T: BorshDeserialize, const L: usize, const U: usize, W> BorshDeserialize
        for BoundedVec<T, L, U, W>
    {
        fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
            let len = if U <= usize::from(u8::MAX) {
                usize::from(u8::deserialize_reader(reader)?)
            } else if U <= usize::from(u16::MAX) {
                usize::from(u16::deserialize_reader(reader)?)
            } else {
                let len = u32::deserialize_reader(reader)?;
                usize::try_from(len).map_err(|_| {
                    borsh::io::Error::new(
                        borsh::io::ErrorKind::Other,
                        alloc::format!("Length overflow: got {}", len),
                    )
                })?
            };
            if len < L {
                return Err(borsh::io::Error::new(
                    borsh::io::ErrorKind::Other,
                    alloc::format!("Lower bound violation: got {} (expected >= {})", len, L),
                ));
            } else if len > U {
                return Err(borsh::io::Error::new(
                    borsh::io::ErrorKind::Other,
                    alloc::format!("Upper bound violation: got {} (expected <= {})", len, U),
                ));
            }
            // adapted from internals for borsh-rs
            let data = if len == 0 {
                Vec::new()
            } else if let Some(vec_bytes) = T::vec_from_reader(len as u32, reader)? {
                vec_bytes
            } else {
                let el_size = core::mem::size_of::<T>() as u32;
                let cautious =
                    core::cmp::max(core::cmp::min(len as u32, 4096 / el_size), 1) as usize;

                // TODO(16): return capacity allocation when we can safely do that.
                let mut result = Vec::with_capacity(cautious);
                for _ in 0..len {
                    result.push(T::deserialize_reader(reader)?);
                }
                result
            };

            Ok(Self {
                inner: data,
                _marker: core::marker::PhantomData,
            })
        }
    }

    impl<T: BorshSchema, const L: usize, const U: usize> BorshSchema for BoundedVec<T, L, U> {
        fn add_definitions_recursively(
            definitions: &mut BTreeMap<borsh::schema::Declaration, borsh::schema::Definition>,
        ) {
            let len_width = if U <= usize::from(u8::MAX) {
                1
            } else if U <= usize::from(u16::MAX) {
                2
            } else {
                4 // proven by design
            };

            let definition = borsh::schema::Definition::Sequence {
                length_width: len_width,
                #[expect(clippy::expect_used)]
                length_range: core::ops::RangeInclusive::<u64>::new(
                    u64::try_from(L).expect("proved by design"),
                    u64::try_from(U).expect("proved by design"),
                ),
                elements: T::declaration(),
            };
            match definitions.entry(Self::declaration()) {
                Entry::Occupied(occ) => {
                    let existing_def = occ.get();
                    assert_eq!(
                    existing_def,
                    &definition,
                    "Redefining type schema for {}. Types with the same names are not supported.",
                    occ.key()
                );
                }
                Entry::Vacant(vac) => {
                    vac.insert(definition);
                }
            }
            T::add_definitions_recursively(definitions);
        }

        fn declaration() -> borsh::schema::Declaration {
            alloc::format!("BoundedVec<{}, {}, {}>", T::declaration(), L, U)
        }
    }

    #[cfg(test)]
    mod tests {
        use borsh::schema::BorshSchemaContainer;

        use super::*;
        #[test]
        #[allow(clippy::expect_used)]
        fn borsh_encdec() {
            let data: BoundedVec<u8, 2, 8> = vec![1u8, 2].try_into().expect("borsh works");
            let buf = &mut Vec::new();
            data.serialize(buf).expect("borsh works");
            let decoded =
                BoundedVec::<u8, 2, 8>::deserialize(&mut buf.as_slice()).expect("borsh works");
            let compatible_decoded =
                BoundedVec::<u8, 1, 255>::deserialize(&mut buf.as_slice()).expect("borsh works");
            assert_eq!(data.get(0), decoded.get(0));
            assert_eq!(data.get(1), decoded.get(1));
            assert_eq!(data.get(0), compatible_decoded.get(0));
            assert_eq!(data.get(1), compatible_decoded.get(1));
            assert!(BoundedVec::<u8, 1, 257>::deserialize(&mut buf.as_slice()).is_err());

            let schema = BorshSchemaContainer::for_type::<BoundedVec<u8, 2, 8>>();
            let schema = schema
                .get_definition("BoundedVec<u8, 2, 8>")
                .expect("borsh works");
            assert!(matches!(
                schema,
                borsh::schema::Definition::Sequence {
                    length_width: 1,
                    ..
                }
            ));
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
mod arbitrary {

    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::Arbitrary;
    use proptest::prelude::*;
    use proptest::strategy::BoxedStrategy;

    impl<T: Arbitrary, const L: usize, const U: usize> Arbitrary
        for BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>
    where
        T::Strategy: 'static,
    {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(any::<T>(), L..=U)
                .prop_map(|items| Self::from_vec(items).unwrap())
                .boxed()
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Serialize};

    // direct impl to unify serde in one place instead of doing attribute on declaration and deserialize here
    impl<T: Serialize, const L: usize, const U: usize> Serialize for BoundedVec<T, L, U> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de, T: Deserialize<'de>, const L: usize, const U: usize> Deserialize<'de>
        for BoundedVec<T, L, U>
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let inner = Vec::<T>::deserialize(deserializer)?;
            if inner.len() < L {
                return Err(serde::de::Error::custom(alloc::format!(
                    "Lower bound violation: got {} (expected >= {})",
                    inner.len(),
                    L
                )));
            } else if inner.len() > U {
                return Err(serde::de::Error::custom(alloc::format!(
                    "Upper bound violation: got {} (expected <= {})",
                    inner.len(),
                    U
                )));
            };
            Ok(BoundedVec {
                inner,
                _marker: core::marker::PhantomData,
            })
        }
    }

    #[cfg(feature = "schema")]
    mod schema {
        use super::*;
        use schemars::schema::{InstanceType, SchemaObject};
        use schemars::JsonSchema;

        // we cannot use `serde` attributes, because these do not work with `const`, only numeric literals supported
        impl<T: JsonSchema, const L: usize, const U: usize, W> JsonSchema for BoundedVec<T, L, U, W> {
            fn schema_name() -> alloc::string::String {
                alloc::format!("BoundedVec{}Min{}Max{}", T::schema_name(), L, U)
            }

            fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::Array.into()),
                    array: Some(alloc::boxed::Box::new(schemars::schema::ArrayValidation {
                        items: Some(schemars::schema::SingleOrVec::Single(
                            T::json_schema(gen).into(),
                        )),
                        #[expect(clippy::expect_used)] // design time failure
                        min_items: Some(
                            u32::try_from(L).expect("JSON schema does not support so large ranges"),
                        ),
                        #[expect(clippy::expect_used)] // design time failure
                        max_items: Some(
                            u32::try_from(U).expect("JSON schema does not support so large ranges"),
                        ),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into()
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use schemars::schema_for;
            #[test]
            fn json_schema() {
                let schema = schema_for!(BoundedVec<u8, 2, 8>);
                let min_items = schema.schema.array.as_ref().unwrap().min_items.unwrap();
                let max_items = schema.schema.array.as_ref().unwrap().max_items.unwrap();
                assert_eq!(min_items, 2);
                assert_eq!(max_items, 8);
            }
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use core::convert::TryInto;

    use super::*;

    #[test]
    fn from_vec() {
        assert!(BoundedVec::<u8, 2, 8>::from_vec(vec![1, 2]).is_ok());
        assert!(BoundedVec::<u8, 2, 8>::from_vec(vec![]).is_err());
        assert!(BoundedVec::<u8, 3, 8>::from_vec(vec![1, 2]).is_err());
        assert!(BoundedVec::<u8, 1, 2>::from_vec(vec![1, 2, 3]).is_err());
    }

    #[test]
    fn is_empty() {
        let data: EmptyBoundedVec<_, 8> = vec![1u8, 2].try_into().unwrap();
        assert!(!data.is_empty());
    }

    #[test]
    fn as_vec() {
        let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
        assert_eq!(data.as_vec(), &vec![1u8, 2]);
    }

    #[test]
    fn as_slice() {
        let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
        assert_eq!(data.as_slice(), &[1u8, 2]);
    }

    #[test]
    fn len() {
        let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
        assert_eq!(data.len(), 2);
    }

    #[test]
    fn first() {
        let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
        assert_eq!(data.first(), &1u8);
    }

    #[test]
    fn last() {
        let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
        assert_eq!(data.last(), &2u8);
    }

    #[test]
    fn mapped() {
        let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
        let data = data.mapped(|x| x * 2);
        assert_eq!(data, [2u8, 4].into());
    }

    #[test]
    fn mapped_ref() {
        let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
        let data = data.mapped_ref(|x| x * 2);
        assert_eq!(data, [2u8, 4].into());
    }

    #[test]
    fn get() {
        let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
        assert_eq!(data.get(1).unwrap(), &2u8);
        assert!(data.get(3).is_none());
    }

    #[test]
    fn try_mapped() {
        let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
        let data = data.try_mapped(|x| 100u8.checked_div(x).ok_or("error"));
        assert_eq!(data, Ok([100u8, 50].into()));
    }

    #[test]
    fn try_mapped_error() {
        let data: BoundedVec<u8, 2, 8> = [0u8, 2].into();
        let data = data.try_mapped(|x| 100u8.checked_div(x).ok_or("error"));
        assert_eq!(data, Err("error"));
    }

    #[test]
    fn try_mapped_ref() {
        let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
        let data = data.try_mapped_ref(|x| 100u8.checked_div(*x).ok_or("error"));
        assert_eq!(data, Ok([100u8, 50].into()));
    }

    #[test]
    fn try_mapped_ref_error() {
        let data: BoundedVec<u8, 2, 8> = [0u8, 2].into();
        let data = data.try_mapped_ref(|x| 100u8.checked_div(*x).ok_or("error"));
        assert_eq!(data, Err("error"));
    }

    #[test]
    fn split_last() {
        let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
        assert_eq!(data.split_last(), (&2u8, [1u8].as_ref()));
        let data1: BoundedVec<_, 1, 8> = vec![1u8].try_into().unwrap();
        assert_eq!(data1.split_last(), (&1u8, Vec::new().as_ref()));
    }

    #[test]
    fn enumerated() {
        let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
        let expected: BoundedVec<_, 2, 8> = vec![(0, 1u8), (1, 2)].try_into().unwrap();
        assert_eq!(data.enumerated(), expected);
    }

    #[test]
    fn into_iter() {
        let mut vec = vec![1u8, 2];
        let mut data: BoundedVec<_, 2, 8> = vec.clone().try_into().unwrap();
        assert_eq!(data.clone().into_iter().collect::<Vec<u8>>(), vec);
        assert_eq!(
            data.iter().collect::<Vec<&u8>>(),
            vec.iter().collect::<Vec<&u8>>()
        );
        assert_eq!(
            data.iter_mut().collect::<Vec<&mut u8>>(),
            vec.iter_mut().collect::<Vec<&mut u8>>()
        );
    }
}

#[cfg(feature = "arbitrary")]
#[cfg(test)]
#[allow(clippy::len_zero)]
mod arb_tests {

    use super::*;
    use alloc::format;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn const_bounded_collections_length_bounded(v: BoundedVec<u8, 1, 2>) {
            prop_assert!(1 <= v.len() && v.len() <= 2);
        }
    }
}
