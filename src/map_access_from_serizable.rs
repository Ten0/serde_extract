use super::*;

pub struct ThisMapAccess<'s, S: Serialize + ?Sized> {
	serializable: &'s S,
	fields: &'static [&'static str],
	start_idx: usize,
}

impl<'s, S: Serialize + ?Sized> ThisMapAccess<'s, S> {
	pub(super) fn new(serializable: &'s S, fields: &'static [&'static str]) -> Self {
		Self {
			serializable,
			fields,
			start_idx: 0,
		}
	}
}

impl<'de, 's, S: Serialize + ?Sized> MapAccess<'de> for ThisMapAccess<'s, S> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: DeserializeSeed<'de>,
	{
		self.fields
			.first()
			.map(|&field_name| seed.deserialize(value::BorrowedStrDeserializer::new(field_name)))
			.transpose()
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		match self.serializable.serialize(ExtractFieldByNameSerializer {
			key_to_find: self
				.fields
				.first()
				.ok_or_else(|| Error::custom("Called next_value_seed without next_key_seed"))?,
			vseed: seed,
			start_idx: self.start_idx,
		})? {
			ExtractFieldByNameState::NotSeen(seed) | ExtractFieldByNameState::ShouldTakeNext(seed) => {
				// If it's an option this will `visit_none`
				self.fields = &self.fields[1..];
				self.start_idx = 0;
				seed.deserialize(value::UnitDeserializer::new())
			}
			ExtractFieldByNameState::Seen(value) => {
				self.fields = &self.fields[1..];
				self.start_idx = 0;
				Ok(value)
			}
			ExtractFieldByNameState::SeenAndMoreOfTheSameAreAvailable {
				value,
				first_next_available,
			} => {
				self.start_idx = first_next_available;
				Ok(value)
			}
			ExtractFieldByNameState::Broken => {
				return Err(Error::custom(
					"Should not happen unless we exited with an error\
            in which case we shouldn't reach this path",
				))
			}
		}
	}

	fn next_entry_seed<K, V>(&mut self, kseed: K, mut vseed: V) -> Result<Option<(K::Value, V::Value)>, Self::Error>
	where
		K: DeserializeSeed<'de>,
		V: DeserializeSeed<'de>,
	{
		Ok(loop {
			break match self.fields.first() {
				Some(&field_name) => {
					match self.serializable.serialize(ExtractFieldByNameSerializer {
						key_to_find: field_name,
						vseed,
						start_idx: self.start_idx,
					})? {
						ExtractFieldByNameState::NotSeen(seed) | ExtractFieldByNameState::ShouldTakeNext(seed) => {
							vseed = seed;
							self.fields = &self.fields[1..];
							self.start_idx = 0;
							continue;
						}
						ExtractFieldByNameState::Seen(value) => {
							self.fields = &self.fields[1..];
							self.start_idx = 0;
							Some((
								kseed.deserialize(value::BorrowedStrDeserializer::new(field_name))?,
								value,
							))
						}
						ExtractFieldByNameState::SeenAndMoreOfTheSameAreAvailable {
							value,
							first_next_available,
						} => {
							self.start_idx = first_next_available;
							Some((
								kseed.deserialize(value::BorrowedStrDeserializer::new(field_name))?,
								value,
							))
						}
						ExtractFieldByNameState::Broken => {
							return Err(Error::custom(
								"Should not happen unless we exited with an error\
                            in which case we shouldn't reach this path",
							))
						}
					}
				}
				None => None,
			};
		})
	}
}

pub struct ExtractFieldByNameSerializer<'de, S> {
	key_to_find: &'de str,
	vseed: S,
	start_idx: usize,
}
pub enum ExtractFieldByNameState<Seed, Val> {
	NotSeen(Seed),
	ShouldTakeNext(Seed),
	Seen(Val),
	SeenAndMoreOfTheSameAreAvailable {
		value: Val,
		/// Will be turned into start_idx for the next iteration
		first_next_available: usize,
	},
	Broken,
}
impl<'de, S: DeserializeSeed<'de>> Serializer for ExtractFieldByNameSerializer<'de, S> {
	type Ok = ExtractFieldByNameState<S, S::Value>;
	type Error = Error;

	type SerializeMap = ExtractFieldByNameSerializeStructOrMap<'de, S>;
	fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		Ok(ExtractFieldByNameSerializeStructOrMap {
			key_to_find: self.key_to_find,
			start_idx: self.start_idx,
			state: ExtractFieldByNameState::NotSeen(self.vseed),
			current_idx: 0,
		})
	}

	type SerializeStruct = ExtractFieldByNameSerializeStructOrMap<'de, S>;
	fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct, Self::Error> {
		Ok(ExtractFieldByNameSerializeStructOrMap {
			key_to_find: self.key_to_find,
			start_idx: self.start_idx,
			state: ExtractFieldByNameState::NotSeen(self.vseed),
			current_idx: 0,
		})
	}

	serializer_unsupported! {
		err = (<Self::Error as serde::ser::Error>::custom("Can only extract from map and struct"));
		bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str bytes none some unit unit_struct
		unit_variant newtype_struct newtype_variant seq tuple tuple_struct tuple_variant
		struct_variant i128 u128
	}
}

pub struct ExtractFieldByNameSerializeStructOrMap<'de, S: DeserializeSeed<'de>> {
	key_to_find: &'de str,
	state: ExtractFieldByNameState<S, S::Value>,
	current_idx: usize,
	start_idx: usize,
}

impl<'de, S: DeserializeSeed<'de>> SerializeStruct for ExtractFieldByNameSerializeStructOrMap<'de, S> {
	type Ok = ExtractFieldByNameState<S, S::Value>;
	type Error = Error;

	fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		if self.current_idx >= self.start_idx {
			if key == self.key_to_find {
				self.state = match std::mem::replace(&mut self.state, ExtractFieldByNameState::Broken) {
					ExtractFieldByNameState::NotSeen(seed) => ExtractFieldByNameState::Seen(
						seed.deserialize(DeserializerFromSerializable { serializable: value })?,
					),
					ExtractFieldByNameState::Seen(value) => ExtractFieldByNameState::SeenAndMoreOfTheSameAreAvailable {
						value,
						first_next_available: self.current_idx,
					},
					more_of_same @ ExtractFieldByNameState::SeenAndMoreOfTheSameAreAvailable { .. } => more_of_same,
					ExtractFieldByNameState::Broken => {
						return Err(Error::custom(
							"ExtractFieldByNameState shouldn't be left in Broken \
                        state unless we exited with an error, \
                        in which case we expect this function to not be called again",
						))
					}
					ExtractFieldByNameState::ShouldTakeNext(_) => {
						return Err(Error::custom(
							"ExtractFieldByNameState should never enter ShouldTakeNext state \
                            through SerializeStruct",
						))
					}
				}
			} else if self.start_idx != 0 && self.current_idx == self.start_idx {
				return Err(Error::custom("Inconsistent serialization is not supported"));
			}
		}
		self.current_idx += 1;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.state)
	}
}

impl<'de, S: DeserializeSeed<'de>> SerializeMap for ExtractFieldByNameSerializeStructOrMap<'de, S> {
	type Ok = ExtractFieldByNameState<S, S::Value>;
	type Error = Error;

	fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		if self.current_idx >= self.start_idx {
			if key.serialize(StringComparisonSerializer {
				check_if_equals: self.key_to_find,
			})? {
				self.state = match std::mem::replace(&mut self.state, ExtractFieldByNameState::Broken) {
					ExtractFieldByNameState::NotSeen(seed) | ExtractFieldByNameState::ShouldTakeNext(seed) => {
						ExtractFieldByNameState::ShouldTakeNext(seed)
					}
					ExtractFieldByNameState::Seen(value) => ExtractFieldByNameState::SeenAndMoreOfTheSameAreAvailable {
						value,
						first_next_available: self.current_idx,
					},
					more_of_same @ ExtractFieldByNameState::SeenAndMoreOfTheSameAreAvailable { .. } => more_of_same,
					ExtractFieldByNameState::Broken => {
						return Err(Error::custom(
							"ExtractFieldByNameState shouldn't be left in Broken \
                        state unless we exited with an error, \
                        in which case we expect this function to not be called again",
						))
					}
				}
			} else if self.start_idx != 0 && self.current_idx == self.start_idx {
				return Err(Error::custom("Inconsistent serialization is not supported"));
			}
		}
		self.current_idx += 1;
		Ok(())
	}

	fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		self.state = match std::mem::replace(&mut self.state, ExtractFieldByNameState::Broken) {
			ExtractFieldByNameState::ShouldTakeNext(seed) => {
				ExtractFieldByNameState::Seen(seed.deserialize(DeserializerFromSerializable { serializable: value })?)
			}
			ExtractFieldByNameState::Broken => {
				return Err(Error::custom(
					"ExtractFieldByNameState shouldn't be left in Broken \
                        state unless we exited with an error, \
                        in which case we expect this function to not be called again",
				))
			}
			other => other,
		};
		Ok(())
	}

	fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
	where
		K: Serialize,
		V: Serialize,
	{
		if self.current_idx >= self.start_idx {
			if key.serialize(StringComparisonSerializer {
				check_if_equals: self.key_to_find,
			})? {
				self.state = match std::mem::replace(&mut self.state, ExtractFieldByNameState::Broken) {
					ExtractFieldByNameState::NotSeen(seed) => ExtractFieldByNameState::Seen(
						seed.deserialize(DeserializerFromSerializable { serializable: value })?,
					),
					ExtractFieldByNameState::Seen(value) => ExtractFieldByNameState::SeenAndMoreOfTheSameAreAvailable {
						value,
						first_next_available: self.current_idx,
					},
					more_of_same @ ExtractFieldByNameState::SeenAndMoreOfTheSameAreAvailable { .. } => more_of_same,
					ExtractFieldByNameState::Broken => {
						return Err(Error::custom(
							"ExtractFieldByNameState shouldn't be left in Broken \
                        state unless we exited with an error, \
                        in which case we expect this function to not be called again",
						))
					}
					ExtractFieldByNameState::ShouldTakeNext(_) => {
						return Err(Error::custom(
							"ExtractFieldByNameState should never enter ShouldTakeNext state",
						))
					}
				};
			} else if self.start_idx != 0 && self.current_idx == self.start_idx {
				return Err(Error::custom("Inconsistent serialization is not supported"));
			}
		}
		self.current_idx += 1;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.state)
	}
}

struct StringComparisonSerializer<'a> {
	check_if_equals: &'a str,
}
impl Serializer for StringComparisonSerializer<'_> {
	type Ok = bool;
	type Error = serde::de::value::Error;
	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		Ok(v == self.check_if_equals)
	}

	serializer_unsupported! {
		err = (<Self::Error as serde::ser::Error>::custom("StringComparisonSerializer only supports comparison through serialize_str"));
		bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char bytes none some unit unit_struct
		unit_variant newtype_struct newtype_variant seq tuple tuple_struct tuple_variant map struct
		struct_variant i128 u128
	}
}
