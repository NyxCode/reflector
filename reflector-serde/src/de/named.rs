use reflector::{Cons, NamedFields, NamedStruct, SizedStruct, Struct};
use serde::de::{Error, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::marker::PhantomData;

struct FieldIndex<T> {
    idx: usize,
    _marker: PhantomData<T>,
}

struct VisitFieldIndex<T>(PhantomData<T>);
impl<'de, T> Visitor<'de> for VisitFieldIndex<T>
where
    T: Struct,
    T::Fields: NamedFields,
{
    type Value = FieldIndex<T>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("field identifier")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let idx = usize::try_from(v).unwrap_or(usize::MAX);
        Ok(FieldIndex {
            idx,
            _marker: PhantomData,
        })
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_bytes(v.as_bytes())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldIndex {
            idx: T::Fields::NAMES
                .iter()
                .position(|name| name.as_bytes() == v)
                .unwrap_or(usize::MAX),
            _marker: PhantomData,
        })
    }
}

impl<'de, T> Deserialize<'de> for FieldIndex<T>
where
    T: Struct,
    T::Fields: NamedFields,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(VisitFieldIndex(PhantomData))
    }
}

// ---

pub struct Visit<'de, T>(pub PhantomData<(&'de (), T)>);

impl<'de, T> Visitor<'de> for Visit<'de, T>
where
    T: NamedStruct + SizedStruct,
    super::tuple::Visit<'de, T>: Visitor<'de, Value = T::Root>,
    T::FieldTypes: Wrap<List: DeserializeFields<'de>>,
{
    type Value = T::Root;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("struct ")?;
        formatter.write_str(T::IDENT)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        super::tuple::Visit(PhantomData).visit_seq(seq)
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut fields = <T::FieldTypes as Wrap>::List::default();
        while let Some(key) = map.next_key::<FieldIndex<T>>()? {
            fields.deserialize(key.idx, &mut map)?;
        }
        let fields = fields.unwrap_all()?;
        Ok(T::from_values(fields))
    }
}

trait DeserializeFields<'de> {
    fn deserialize<A: MapAccess<'de>>(&mut self, idx: usize, map: &mut A) -> Result<(), A::Error>;
}

impl<'de> DeserializeFields<'de> for () {
    fn deserialize<A: MapAccess<'de>>(&mut self, _: usize, map: &mut A) -> Result<(), A::Error> {
        map.next_value::<IgnoredAny>()?;
        Ok(())
    }
}

impl<'de, Head, Tail> DeserializeFields<'de> for Cons<Option<Head>, Tail>
where
    Head: Deserialize<'de>,
    Tail: DeserializeFields<'de>,
{
    fn deserialize<A: MapAccess<'de>>(&mut self, idx: usize, map: &mut A) -> Result<(), A::Error> {
        if idx > 0 {
            return Tail::deserialize(&mut self.1, idx - 1, map);
        }
        if self.0.is_some() {
            return Err(A::Error::custom("duplicate field"));
        }

        self.0 = Some(map.next_value()?);
        Ok(())
    }
}

trait Wrap {
    type List: Unwrap<List = Self> + Default;
}
impl Wrap for () {
    type List = ();
}
impl<Head, Tail> Wrap for Cons<Head, Tail>
where
    Tail: Wrap,
{
    type List = Cons<Option<Head>, Tail::List>;
}

trait Unwrap {
    type List;

    fn unwrap_all<Error: serde::de::Error>(self) -> Result<Self::List, Error>;
}
impl Unwrap for () {
    type List = ();

    fn unwrap_all<Error>(self) -> Result<Self::List, Error> {
        Ok(())
    }
}
impl<Head, Tail> Unwrap for Cons<Option<Head>, Tail>
where
    Tail: Unwrap,
{
    type List = Cons<Head, Tail::List>;

    fn unwrap_all<Error: serde::de::Error>(self) -> Result<Self::List, Error> {
        Ok(Cons(
            self.0.ok_or_else(|| Error::custom("missing field"))?,
            self.1.unwrap_all()?,
        ))
    }
}
