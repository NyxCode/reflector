use reflector::{Cons, SizedStruct};
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::marker::PhantomData;

pub struct Visit<'de, T>(pub PhantomData<(&'de (), T)>);

impl<'de, T, Inner> Visitor<'de> for Visit<'de, T>
where
    T: SizedStruct<FieldTypes = Cons<Inner, ()>>,
    Inner: Deserialize<'de>,
    super::tuple::Visit<'de, T>: Visitor<'de, Value = T::Root>,
{
    type Value = T::Root;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("tuple struct ")?;
        formatter.write_str(T::IDENT)
    }

    fn visit_newtype_struct<D: Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        Ok(T::from_values(Cons(Inner::deserialize(de)?, ())))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        super::tuple::Visit(PhantomData).visit_seq(seq)
    }
}
