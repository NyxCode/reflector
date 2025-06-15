use std::{fmt::Formatter, marker::PhantomData};

use reflector::{
    Cons, Enum, Field, List, NamedFieldList, NamedShape, SizedStruct, TupleShape, UnitShape,
    Variant, VariantList,
};
use serde::{
    Deserialize, Deserializer,
    de::{EnumAccess, Error, VariantAccess, Visitor},
};

struct Discriminant<T>(usize, PhantomData<T>);

struct VisitDiscriminant<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for VisitDiscriminant<T>
where
    T: Enum,
{
    type Value = Discriminant<T>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("variant identifier")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match usize::try_from(v).map_err(E::custom)? {
            idx if idx < T::Variants::LENGTH => Ok(Discriminant(idx, PhantomData)),
            _ => Err(E::custom("variant index out of range")),
        }
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
        Ok(Discriminant(
            T::Variants::NAMES
                .iter()
                .position(|name| name.as_bytes() == v)
                .ok_or_else(|| E::custom("unknown variant"))?,
            PhantomData,
        ))
    }
}

impl<'de, T> Deserialize<'de> for Discriminant<T>
where
    T: Enum,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(VisitDiscriminant(PhantomData))
    }
}

pub struct Visit<'de, T>(pub PhantomData<(&'de (), T)>);

impl<'de, T> Visitor<'de> for Visit<'de, T>
where
    T: Enum<Variants: DeserializeVariants<'de, T>>,
{
    type Value = T;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("enum ")?;
        formatter.write_str(T::IDENT)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let (Discriminant(idx, ..), data) = data.variant::<Discriminant<T>>()?;
        <T::Variants as DeserializeVariants<'de, T>>::deserialize(idx, data)
    }
}

trait DeserializeVariant<'de, T, Shape, Fields> {
    fn deserialize<A: VariantAccess<'de>>(v: A) -> Result<T, A::Error>;
}

// unit variant
impl<'de, T, V> DeserializeVariant<'de, T, UnitShape, ()> for V
where
    V: Variant<Shape = UnitShape, Root = T> + SizedStruct<FieldTypes = ()>,
{
    fn deserialize<A: VariantAccess<'de>>(v: A) -> Result<T, A::Error> {
        v.unit_variant().map(|_| V::from_values(()))
    }
}
// empty tuple variant
impl<'de, T, V> DeserializeVariant<'de, T, TupleShape, ()> for V
where
    V: Variant<Shape = NamedShape, Root = T> + SizedStruct,
    super::tuple::Visit<'de, V>: Visitor<'de, Value = T>,
{
    fn deserialize<A: VariantAccess<'de>>(v: A) -> Result<T, A::Error> {
        v.tuple_variant(0, super::tuple::Visit::<V>(PhantomData))
    }
}
// newtype variant
impl<'de, T, V, InnerField> DeserializeVariant<'de, T, TupleShape, Cons<InnerField, ()>> for V
where
    InnerField: Field<Type: Sized + Deserialize<'de>>,
    V: Variant<Shape = TupleShape, Root = T> + SizedStruct<FieldTypes = Cons<InnerField::Type, ()>>,
    super::tuple::Visit<'de, V>: Visitor<'de, Value = T>,
{
    fn deserialize<A: VariantAccess<'de>>(v: A) -> Result<T, A::Error> {
        v.newtype_variant::<InnerField::Type>()
            .map(|inner| V::from_values(Cons(inner, ())))
    }
}
// tuple variant
impl<'de, T, V, F0, F1, Fs> DeserializeVariant<'de, T, TupleShape, Cons<F0, Cons<F1, Fs>>> for V
where
    V: Variant<Shape = TupleShape, Root = T> + SizedStruct,
    super::tuple::Visit<'de, V>: Visitor<'de, Value = T>,
{
    fn deserialize<A: VariantAccess<'de>>(v: A) -> Result<T, A::Error> {
        v.tuple_variant(0, super::tuple::Visit::<V>(PhantomData))
    }
}
// struct variant
impl<'de, T, V, Fields> DeserializeVariant<'de, T, NamedShape, Fields> for V
where
    V: Variant<Shape = NamedShape, Root = T, Fields: NamedFieldList> + SizedStruct,
    super::named::Visit<'de, V>: Visitor<'de, Value = T>,
{
    fn deserialize<A: VariantAccess<'de>>(v: A) -> Result<T, A::Error> {
        v.struct_variant(V::Fields::NAMES, super::named::Visit::<V>(PhantomData))
    }
}

trait DeserializeVariants<'de, T> {
    fn deserialize<A: VariantAccess<'de>>(idx: usize, v: A) -> Result<T, A::Error>;
}

impl<'de, T> DeserializeVariants<'de, T> for () {
    fn deserialize<A: VariantAccess<'de>>(_: usize, _: A) -> Result<T, A::Error> {
        unreachable!()
    }
}

impl<'de, T, Head, Tail> DeserializeVariants<'de, T> for Cons<Head, Tail>
where
    Head: Variant + DeserializeVariant<'de, T, Head::Shape, Head::Fields>,
    Tail: DeserializeVariants<'de, T>,
{
    fn deserialize<A: VariantAccess<'de>>(idx: usize, v: A) -> Result<T, A::Error> {
        if idx > 0 {
            return Tail::deserialize(idx - 1, v);
        }
        Head::deserialize(v)
    }
}
