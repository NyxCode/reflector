use reflector::{EnumType, Field, FieldValues, FromValues, Introspect, Struct, StructType, ValuesOf};
use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::marker::PhantomData;

struct _Field<T> {
    idx: usize,
    _marker: PhantomData<T>,
}

struct _FieldVisitor<T>(PhantomData<T>);
impl<'de, T> Visitor<'de> for _FieldVisitor<T>
where
    T: Struct,
    T::Fields: Fields,
{
    type Value = _Field<T>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("field identifier")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let idx = usize::try_from(v).unwrap_or(usize::MAX);
        Ok(_Field {
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
        Ok(_Field {
            idx: T::Fields::field_index(v).unwrap_or(usize::MAX),
            _marker: PhantomData,
        })
    }
}

impl<'de, T> Deserialize<'de> for _Field<T>
where
    T: Struct,
    T::Fields: Fields,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(_FieldVisitor(PhantomData))
    }
}

// ---

struct _Visit<'de, T>(PhantomData<(&'de (), T)>);

impl<'de, T> Visitor<'de> for _Visit<'de, T>
where
    T: Struct + FromValues,
    T::Fields: FieldValues,
    ValuesOf<T::Fields>: FromSequence<'de>,
{
    type Value = T;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("struct ")?;
        formatter.write_str(T::IDENT)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let values = ValuesOf::<T::Fields>::from_sequence(seq)?;
        Ok(T::from_values(values))
    }
}

struct Reflect<T>(pub T);
impl<'de, T> Deserialize<'de> for Reflect<T> where T: Struct {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        //deserializer.deserialize_struct(T::IDENT, &[], _Visit(PhantomData))
        unimplemented!()
    }
}







trait FromSequence<'de>: Sized {
    fn from_sequence<Seq>(seq: Seq) -> Result<Self, Seq::Error>
    where
        Seq: SeqAccess<'de>;
}
impl<'de> FromSequence<'de> for () {
    fn from_sequence<Seq>(seq: Seq) -> Result<Self, Seq::Error>
    where
        Seq: SeqAccess<'de>,
    {
        Ok(())
    }
}
impl<'de, Head, Tail> FromSequence<'de> for (Head, Tail)
where
    Head: Deserialize<'de>,
    Tail: FromSequence<'de>,
{
    fn from_sequence<Seq>(mut seq: Seq) -> Result<Self, Seq::Error>
    where
        Seq: SeqAccess<'de>,
    {
        Ok((
            seq.next_element()?
                .ok_or_else(|| Seq::Error::custom("not enough items in sequence"))?,
            Tail::from_sequence(seq)?,
        ))
    }
}

trait Fields {
    fn field_index(ident: &[u8]) -> Option<usize> {
        None
    }
}

impl Fields for () {}

impl<Head, Tail> Fields for (Head, Tail)
where
    Head: Field,
    Tail: Fields,
{
    fn field_index(ident: &[u8]) -> Option<usize> {
        match Head::IDENT {
            Some(i) if i.as_bytes() == ident => Some(Head::INDEX as _),
            _ => Tail::field_index(ident),
        }
    }
}


#[test]
fn works() {
    #[derive(Introspect)]
    struct X {
        a: i32
    }
    
    //let result: X = serde_json::from_str("{\"a\": 42}").unwrap();
}