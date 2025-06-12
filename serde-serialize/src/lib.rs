use reflector::*;
use serde::ser::{
    SerializeStruct, SerializeStructVariant, SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};

struct Reflect<'a, T>(&'a T);

impl<'a, T> Serialize for Reflect<'a, T>
where
    T: HasShape,
    T: ReflectType<T::Shape>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        T::serialize(&self.0, serializer)
    }
}

trait ReflectType<Shape> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

trait ReflectStruct<StructShape> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<T> ReflectType<StructShape> for T
where
    T: Struct + ReflectStruct<T::StructShape>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <T as ReflectStruct<_>>::serialize(self, serializer)
    }
}



impl<T> ReflectStruct<NamedStructShape> for T
where
    T: Struct,
    T::Fields: RecurseNamedFields<T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        T::Fields::serialize(self, serializer.serialize_struct(T::IDENT, T::Fields::LEN)?)
    }
}

trait SerdeSerializeStruct<const VARIANT: bool> {
    type Ok;
    type Error: std::error::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;
    #[inline]
    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        let _ = key;
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

impl<S: SerializeStruct> SerdeSerializeStruct<false> for S {
    type Ok = <S as SerializeStruct>::Ok;
    type Error = <S as SerializeStruct>::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        <S as SerializeStruct>::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <S as SerializeStruct>::end(self)
    }
}

impl<S: SerializeStructVariant> SerdeSerializeStruct<true> for S {
    type Ok = <S as SerializeStructVariant>::Ok;
    type Error = <S as SerializeStructVariant>::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        <S as SerializeStructVariant>::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <S as SerializeStructVariant>::end(self)
    }
}

trait SerdeSerializeTuples<const VARIANT: bool> {
    type Ok;
    type Error: std::error::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

impl<S: SerializeTupleStruct> SerdeSerializeTuples<false> for S {
    type Ok = <S as SerializeTupleStruct>::Ok;
    type Error = <S as SerializeTupleStruct>::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        <S as SerializeTupleStruct>::serialize_field(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <S as SerializeTupleStruct>::end(self)
    }
}
impl<S: SerializeTupleVariant> SerdeSerializeTuples<true> for S {
    type Ok = <S as SerializeTupleVariant>::Ok;
    type Error = <S as SerializeTupleVariant>::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        <S as SerializeTupleVariant>::serialize_field(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <S as SerializeTupleVariant>::end(self)
    }
}

trait RecurseNamedFields<Parent> {
    const LEN: usize;

    fn serialize<const VARIANT: bool, S>(parent: &Parent, ctx: S) -> Result<S::Ok, S::Error>
    where
        S: SerdeSerializeStruct<VARIANT>;
}

impl<Parent> RecurseNamedFields<Parent> for () {
    const LEN: usize = 0;

    fn serialize<const VARIANT: bool, S>(parent: &Parent, ctx: S) -> Result<S::Ok, S::Error>
    where
        S: SerdeSerializeStruct<VARIANT>,
    {
        ctx.end()
    }
}

impl<Parent, Head, Tail> RecurseNamedFields<Parent> for (Head, Tail)
where
    Head: Field<Parent = Parent, Ident = &'static str>,
    Head::Type: Serialize,
    Tail: RecurseNamedFields<Parent>,
{
    const LEN: usize = 1 + Tail::LEN;

    fn serialize<const VARIANT: bool, S>(
        parent: &Parent,
        mut serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: SerdeSerializeStruct<VARIANT>,
    {
        serializer.serialize_field(Head::IDENT, Head::try_get_ref(parent).unwrap())?;

        Tail::serialize(parent, serializer)
    }
}

// tuple

impl<T> ReflectStruct<TupleStructShape> for T
where
    T: Struct,
    T::Fields: RecurseTupleFields<T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if T::Fields::LEN == 1 {
            serializer.serialize_newtype_struct(T::IDENT, T::Fields::first(self))
        } else {
            T::Fields::serialize(
                self,
                serializer.serialize_tuple_struct(T::IDENT, T::Fields::LEN)?,
            )
        }
    }
}

trait RecurseTupleFields<Parent> {
    const LEN: usize;
    type TypeOfFirst: Serialize + ?Sized;

    fn first(parent: &Parent) -> &Self::TypeOfFirst;

    fn serialize<const VARIANT: bool, S>(parent: &Parent, ctx: S) -> Result<S::Ok, S::Error>
    where
        S: SerdeSerializeTuples<VARIANT>;
}

impl<Parent> RecurseTupleFields<Parent> for () {
    const LEN: usize = 0;
    type TypeOfFirst = ();

    fn first(parent: &Parent) -> &Self::TypeOfFirst {
        unreachable!() as _
    }

    fn serialize<const VARIANT: bool, S>(parent: &Parent, ctx: S) -> Result<S::Ok, S::Error>
    where
        S: SerdeSerializeTuples<VARIANT>,
    {
        ctx.end()
    }
}

impl<Parent, Head, Tail> RecurseTupleFields<Parent> for (Head, Tail)
where
    Head: Field<Parent = Parent>,
    Head::Type: Serialize,
    Tail: RecurseTupleFields<Parent>,
{
    const LEN: usize = 1 + Tail::LEN;
    type TypeOfFirst = Head::Type;

    fn first(parent: &Parent) -> &Self::TypeOfFirst {
        Head::try_get_ref(parent).unwrap()
    }

    fn serialize<const VARIANT: bool, S>(
        parent: &Parent,
        mut serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: SerdeSerializeTuples<VARIANT>,
    {
        serializer.serialize_field(Head::try_get_ref(parent).unwrap())?;

        Tail::serialize(parent, serializer)
    }
}

// enum

impl<T> ReflectType<EnumShape> for T
where
    T: Enum,
    T::Variants: RecurseVariants<T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        T::Variants::serialize(self, 0, serializer)
    }
}

trait RecurseVariants<Parent> {
    fn serialize<S>(parent: &Parent, index: u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<Parent> RecurseVariants<Parent> for () {
    fn serialize<S>(parent: &Parent, index: u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unreachable!()
    }
}

impl<Parent, Head, Tail> RecurseVariants<Parent> for (Head, Tail)
where
    Head: Variant<Parent = Parent> + ReflectVariant<Parent, Head::StructShape>,
    Tail: RecurseVariants<Parent>,
{
    fn serialize<S>(parent: &Parent, index: u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if Head::is_active(parent) {
            Head::serialize(parent, index, serializer)
        } else {
            Tail::serialize(parent, index + 1, serializer)
        }
    }
}

trait ReflectVariant<Parent, StructShape> {
    fn serialize<S>(parent: &Parent, index: u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<T, Parent> ReflectVariant<Parent, NamedStructShape> for T
where
    Parent: Enum,
    T: Variant<Parent = Parent>,
    T::Fields: RecurseNamedFields<Parent>,
{
    fn serialize<S>(parent: &Parent, index: u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        T::Fields::serialize(
            parent,
            serializer.serialize_struct_variant(Parent::IDENT, index, T::IDENT, T::Fields::LEN)?,
        )
    }
}

impl<T, Parent> ReflectVariant<Parent, TupleStructShape> for T
where
    Parent: Enum,
    T: Variant<Parent = Parent>,
    T::Fields: RecurseTupleFields<Parent>,
{
    fn serialize<S>(parent: &Parent, index: u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if T::Fields::LEN == 1 {
            serializer.serialize_newtype_variant(
                Parent::IDENT,
                index,
                T::IDENT,
                T::Fields::first(parent),
            )
        } else {
            T::Fields::serialize(
                parent,
                serializer.serialize_tuple_variant(
                    Parent::IDENT,
                    index,
                    T::IDENT,
                    T::Fields::LEN,
                )?,
            )
        }
    }
}

#[test]
fn works() {
    #[derive(Reflect)]
    struct A<X> {
        a: i32,
        b: X,
    }

    println!(
        "{}",
        serde_json::to_string(&Reflect(&A { a: 42, b: 3u8 })).unwrap()
    );
    println!(
        "{}",
        serde_json::to_string(&Reflect(&A { a: 42, b: "hey" })).unwrap()
    );

    #[derive(Reflect)]
    struct B<'a>(i32, &'a str);
    println!(
        "{}",
        serde_json::to_string(&Reflect(&B(42, "hey"))).unwrap()
    );

    #[derive(Reflect)]
    enum C<'a> {
        //A,
        B(i32),
        C(i32, &'a str),
        D { x: &'a str },
    };
    println!("{}", serde_json::to_string(&Reflect(&C::B(3))).unwrap());
    println!("{}", serde_json::to_string(&Reflect(&C::C(3, "hey"))).unwrap());
    println!("{}", serde_json::to_string(&Reflect(&C::D { x: "hey" })).unwrap());
}
