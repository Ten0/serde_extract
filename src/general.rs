use super::*;

pub struct SerializerFromVisitor<'de, V> {
	pub(crate) visitor: V,
	pub(crate) _spooky: PhantomData<&'de ()>,
}

impl<'de, V: Visitor<'de>> Serializer for SerializerFromVisitor<'de, V> {
	type Ok = V::Value;
	type Error = Error;

	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_bool(v)
	}

	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_i8(v)
	}

	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_i16(v)
	}

	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_i32(v)
	}

	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_i64(v)
	}

	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_u8(v)
	}

	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_u16(v)
	}

	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_u32(v)
	}

	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_u64(v)
	}

	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_f32(v)
	}

	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_f64(v)
	}

	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_char(v)
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_str(v)
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_bytes(v)
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_none()
	}

	fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: Serialize,
	{
		self.visitor
			.visit_some(DeserializerFromSerializable { serializable: value })
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_unit()
	}

	fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_unit()
	}

	fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> {
		self.visitor.visit_borrowed_str(variant)
	}

	fn serialize_newtype_struct<T: ?Sized>(self, _: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: Serialize,
	{
		self.visitor
			.visit_newtype_struct(DeserializerFromSerializable { serializable: value })
	}

	fn serialize_newtype_variant<T: ?Sized>(
		self,
		_: &'static str,
		_: u32,
		variant: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: Serialize,
	{
		self.visitor
			.visit_enum(newtype_variant::ThisEnumAccess { variant, value })
	}

	serializer_unsupported! {
		err = (<Self::Error as serde::ser::Error>::custom("Deserialization from seq-like serialization is unsupported, unless hinted with deserialize_struct"));
		seq tuple tuple_struct tuple_variant map struct struct_variant
	}
}
