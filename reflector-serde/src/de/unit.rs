use std::{fmt::Formatter, marker::PhantomData};

use reflector::SizedStruct;
use serde::de::{Error, Visitor};

pub struct Visit<'de, T>(pub PhantomData<(&'de (), T)>);

impl<'de, T> Visitor<'de> for Visit<'de, T>
where
    T: SizedStruct<FieldTypes = ()>,
{
    type Value = T::Root;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("unit struct ")?;
        formatter.write_str(T::IDENT)
    }

    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Ok(T::from_values(()))
    }
}
