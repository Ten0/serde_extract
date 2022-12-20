use super::*;

pub struct ThisEnumAccess<'s, S: Serialize + ?Sized> {
	pub(crate) variant: &'static str,
	pub(crate) value: &'s S,
}

impl<'s, 'de, S: Serialize + ?Sized> EnumAccess<'de> for ThisEnumAccess<'s, S> {
	type Error = Error;

	type Variant = ThisVariantAccess<'s, S>;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		Ok((
			seed.deserialize(value::BorrowedStrDeserializer::new(self.variant))?,
			ThisVariantAccess { value: self.value },
		))
	}
}

pub struct ThisVariantAccess<'s, S: Serialize + ?Sized> {
	value: &'s S,
}

impl<'de, S: Serialize + ?Sized> VariantAccess<'de> for ThisVariantAccess<'_, S> {
	type Error = Error;

	fn unit_variant(self) -> Result<(), Self::Error> {
		Err(Error::invalid_type(Unexpected::NewtypeVariant, &"a unit variant"))
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		seed.deserialize(DeserializerFromSerializable {
			serializable: self.value,
		})
	}

	fn tuple_variant<V>(self, _: usize, _: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Err(Error::invalid_type(Unexpected::NewtypeVariant, &"a tuple variant"))
	}

	fn struct_variant<V>(self, _: &'static [&'static str], _: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Err(Error::invalid_type(Unexpected::NewtypeVariant, &"a struct variant"))
	}
}
