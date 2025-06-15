mod r#enum;
mod named;
mod newtype;
mod tuple;
mod unit;

use std::marker::PhantomData;

use reflector::{
    Cons, Enum, EnumKind, Introspect, List, NamedFieldList, NamedShape, NamedStruct, Struct,
    StructKind, TupleShape, UnitShape, VariantList,
};
use serde::{Deserialize, Deserializer, de::Visitor};

trait DeserializeKind<'de, T, Kind>: Sized {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error>;
}
trait DeserializeStruct<'de, T, Shape>: Sized {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error>;
}
trait DeserializeTuple<'de, T, Fields>: Sized {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error>;
}

impl<'de, T> DeserializeKind<'de, T, StructKind> for T
where
    T: Struct + DeserializeStruct<'de, T, T::Shape>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        DeserializeStruct::deserialize(de)
    }
}
impl<'de, T> DeserializeKind<'de, T, EnumKind> for T
where
    T: Enum,
    r#enum::Visit<'de, T>: Visitor<'de, Value = T>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_enum(
            T::IDENT,
            T::Variants::NAMES,
            r#enum::Visit::<T>(PhantomData),
        )
    }
}

impl<'de, T> DeserializeStruct<'de, T, NamedShape> for T
where
    T: NamedStruct,
    named::Visit<'de, T>: Visitor<'de, Value = T>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_struct(T::IDENT, T::Fields::NAMES, named::Visit::<T>(PhantomData))
    }
}
impl<'de, T> DeserializeStruct<'de, T, TupleShape> for T
where
    T: Struct + DeserializeTuple<'de, T, T::Fields>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        DeserializeTuple::deserialize(de)
    }
}
impl<'de, T> DeserializeStruct<'de, T, UnitShape> for T
where
    T: Struct,
    unit::Visit<'de, T>: Visitor<'de, Value = T>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_unit_struct(T::IDENT, unit::Visit(PhantomData))
    }
}

impl<'de, T> DeserializeTuple<'de, T, ()> for T
where
    T: Struct,
    tuple::Visit<'de, T>: Visitor<'de, Value = T>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_tuple_struct(T::IDENT, 0, tuple::Visit(PhantomData))
    }
}
impl<'de, T, Inner> DeserializeTuple<'de, T, Cons<Inner, ()>> for T
where
    T: Struct,
    newtype::Visit<'de, T>: Visitor<'de, Value = T>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_newtype_struct(T::IDENT, newtype::Visit(PhantomData))
    }
}
impl<'de, T, F0, F1, Fs> DeserializeTuple<'de, T, Cons<F0, Cons<F1, Fs>>> for T
where
    T: Struct,
    tuple::Visit<'de, T>: Visitor<'de, Value = T>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_tuple_struct(T::IDENT, T::Fields::LENGTH, tuple::Visit(PhantomData))
    }
}

pub struct Reflect<T>(pub T);

impl<'de, T> Deserialize<'de> for Reflect<T>
where
    T: Introspect,
    T: DeserializeKind<'de, T, T::Kind>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        DeserializeKind::deserialize(de).map(Self)
    }
}
