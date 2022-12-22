//! Enables turning a value that has `Serialize` into a `Deserializer`
//!
//! Effectively this enables extracting a struct that implements `Deserialize` from a struct that
//! implements `Serialize`.
//!
//! # Usage
//! ## TL;DR
//! ```
//! #[derive(serde_derive::Serialize)]
//! struct Source<'a> {
//! 	a: &'a str,
//! 	b: u32,
//! }
//! #[derive(Debug, PartialEq, serde_derive::Deserialize)]
//! struct Extract {
//! 	b: u32,
//! }
//! assert_eq!(
//! 	Extract { b: 3 },
//! 	serde_extract::extract(&Source { a: "hello", b: 3 }).unwrap(),
//! );
//! ```
//!
//! ## More realistic example
//!
//! Let's say we're in a scenario where we want to implement an SDK where results are paginated, we only need to send
//! the original query once, but we need to re-send `page_size` if it was provided in the original query.
//! Since the code that manages pagination has no knowledge of the underlying struct, and because adding a `page_size`
//! argument to our `make_paginated_request` function would be very un-ergonomic because (let's say) it would be very
//! rarely used and it's nicer to specify it in the same struct as the rest of the query parameters, this is a good
//! use-case for this crate.
//!
//! ```
//! // This will be our original query
//! #[derive(serde_derive::Serialize)]
//! struct SomeSpecificRequest {
//! 	field_a: &'static str,
//! 	page_size: usize,
//! }
//!
//! // Let's say make_request is our generic function that makes a call to the server
//! make_paginated_request(&SomeSpecificRequest {
//! 	field_a: "hello!",
//! 	page_size: 2,
//! })
//! .expect("Failed to make request");
//!
//! fn make_paginated_request<S: serde::Serialize>(
//! 	serializable: &S,
//! ) -> Result<(), Box<dyn std::error::Error>> {
//! 	#[derive(serde_derive::Deserialize)]
//! 	struct MaybePageSize {
//! 		page_size: Option<usize>,
//! 	}
//! 	// We will reuse the page size for the subsequent paginated requests if it was
//! 	// provided in the original query, so we need to extract it
//! 	let page_size_for_this_request =
//! 		serde_extract::extract::<MaybePageSize, _>(serializable)?.page_size;
//! 	// this works:
//! 	assert_eq!(page_size_for_this_request, Some(2));
//! 	// Make request...
//! 	Ok(())
//! }
//! ```
//!
//! # Limitations
//!
//! - Sequences are not supported for now (although support could theoritically be added, algorithmic complexity of
//!   generated code would be O(n²) where n is the number of elements in the sequence because we would need to re-drive
//!   the [`Serializer`] for each element called for by the [`Visitor`] through [`MapAccess`])
//! - For the same reason, deserializing into `map`s is currently unsupported. Specifically, currently we can only
//!   extract struct fields if the fields names are hinted by [`deserialize_struct`](Deserializer::deserialize_struct).
//!   (This enables driving the [`Serializer`] only as many times as there are fields to extract. In practice if both
//!   sides are regular structs, the optimizer probably turns that into zero-cost extraction. In theory again, support
//!   for deserializing into maps could be added with O(n²) complexity where n is the number of input fields.)

mod general;
mod map_access_from_serizable;
mod newtype_variant;

use {
	serde::{
		de::{Error as _, *},
		forward_to_deserialize_any,
		ser::*,
	},
	serde_serializer_quick_unsupported::serializer_unsupported,
	std::marker::PhantomData,
};

pub use serde::de::value::Error;

/// Extract a `T: DeserializeOwned` from `S: Serialize`
///
/// See [crate-level documentation](crate) for examples
pub fn extract<T, S>(serializable: &S) -> Result<T, Error>
where
	S: Serialize + ?Sized,
	T: DeserializeOwned,
{
	T::deserialize(DeserializerFromSerializable { serializable })
}

/// Our serializer that can be built from a type that implements `Serialize`
///
/// Note that while it implements `Deserializer<'de>` for any lifetime `'de`, in practice it will never provide
/// you with borrowed values (because these don't exist on [`Serializer`]).
/// This means that attempting to deserialize types that don't implement [`DeserializeOwned`] from this will most likely
/// fail.
pub struct DeserializerFromSerializable<'s, S: Serialize + ?Sized> {
	serializable: &'s S,
}

impl<'s, S: Serialize + ?Sized> DeserializerFromSerializable<'s, S> {
	pub fn new(serializable: &'s S) -> Self {
		Self { serializable }
	}
}

impl<'de, S: Serialize + ?Sized> Deserializer<'de> for DeserializerFromSerializable<'_, S> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.serializable.serialize(general::SerializerFromVisitor::<_, false> {
			visitor,
			_spooky: PhantomData,
		})
	}

	fn deserialize_struct<V>(
		self,
		_: &'static str,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visitor.visit_map(map_access_from_serizable::ThisMapAccess::new(self.serializable, fields))
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.serializable.serialize(general::SerializerFromVisitor::<_, true> {
			visitor,
			_spooky: PhantomData,
		})
	}

	// For now we'll ignore any hint except struct and just propagate what the serializer gives us
	// this may be improved in the future on an as-needed basis
	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf unit unit_struct newtype_struct seq tuple
		tuple_struct map enum identifier ignored_any
	}
}
