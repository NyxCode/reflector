use std::{fmt::Formatter, marker::PhantomData};

use reflector::{Cons, SizedStruct};
use serde::{
    Deserialize,
    de::{Error, SeqAccess, Visitor},
};

pub struct Visit<'de, T>(pub PhantomData<(&'de (), T)>);

impl<'de, T> Visitor<'de> for Visit<'de, T>
where
    T: SizedStruct,
    T::FieldTypes: FromSequence<'de>,
{
    type Value = T::Root;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("tuple struct ")?;
        formatter.write_str(T::IDENT)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        T::FieldTypes::from_sequence(seq).map(T::from_values)
    }
}

trait FromSequence<'de>: Sized {
    fn from_sequence<Seq>(seq: Seq) -> Result<Self, Seq::Error>
    where
        Seq: SeqAccess<'de>;
}
impl<'de> FromSequence<'de> for () {
    fn from_sequence<Seq>(_: Seq) -> Result<Self, Seq::Error>
    where
        Seq: SeqAccess<'de>,
    {
        Ok(())
    }
}
impl<'de, Head, Tail> FromSequence<'de> for Cons<Head, Tail>
where
    Head: Deserialize<'de>,
    Tail: FromSequence<'de>,
{
    fn from_sequence<Seq>(mut seq: Seq) -> Result<Self, Seq::Error>
    where
        Seq: SeqAccess<'de>,
    {
        Ok(Cons(
            seq.next_element()?
                .ok_or_else(|| Seq::Error::custom("not enough items in sequence"))?,
            Tail::from_sequence(seq)?,
        ))
    }
}
