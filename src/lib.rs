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

pub fn extract<'de, T, S>(serializable: &S) -> Result<T, Error>
where
	S: Serialize + ?Sized,
	T: Deserialize<'de>,
{
	T::deserialize(DeserializerFromSerializable { serializable })
}

pub struct DeserializerFromSerializable<'s, S: Serialize + ?Sized> {
	serializable: &'s S,
}

impl<'de, S: Serialize + ?Sized> Deserializer<'de> for DeserializerFromSerializable<'_, S> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.serializable.serialize(general::SerializerFromVisitor {
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

	// For now we'll ignore any hint except struct and just propagate what the serializer gives us
	// this may be improved in the future on an as-needed basis
	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
		bytes byte_buf option unit unit_struct newtype_struct seq tuple
		tuple_struct map enum identifier ignored_any
	}
}
