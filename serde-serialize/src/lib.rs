use reflector::*;
use serde::ser::{
    SerializeStruct, SerializeStructVariant, SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};
use std::ops::ControlFlow;

struct Reflect<'a, T>(&'a T);

// --

trait FieldVisitor<Parent>: Sized {
    type Break;

    fn visit<F>(self, value: &F::Type) -> ControlFlow<Self::Break, Self>
    where
        F: Field<Parent = Parent>,
        F::Type: Serialize;
}

trait Fields<Parent> {
    const LEN: usize = 0;
    const IS_NEWTYPE: bool = false;

    fn for_each<V>(parent: &Parent, visit: V) -> ControlFlow<V::Break, V>
    where
        V: FieldVisitor<Parent>,
    {
        ControlFlow::Continue(visit)
    }
}

impl<Parent> Fields<Parent> for () {}

impl<Parent, Head, Tail> Fields<Parent> for (Head, Tail)
where
    Head: Field<Parent = Parent>,
    Head::Type: Serialize,
    Tail: Fields<Parent>,
{
    const LEN: usize = 1 + Tail::LEN;
    const IS_NEWTYPE: bool = Self::LEN == 1;

    fn for_each<V>(parent: &Parent, visit: V) -> ControlFlow<V::Break, V>
    where
        V: FieldVisitor<Parent>,
    {
        let visit = visit.visit::<Head>(Head::try_get_ref(parent).unwrap())?;
        Tail::for_each(parent, visit)
    }
}

// --

trait VariantVisitor<Parent>: Sized {
    type Break;

    fn visit<T>(self, parent: &Parent) -> ControlFlow<Self::Break, Self>
    where
        T: ReflectStruct<Parent, T::StructShape, <T::Fields as Count>::Count>,
        T: Variant<Parent = Parent>,
        T::Fields: Fields<Parent> + Count;
}

trait Variants<Parent> {
    const LEN: usize = 0;

    fn for_each<V>(parent: &Parent, visit: V) -> ControlFlow<V::Break, V>
    where
        V: VariantVisitor<Parent>,
    {
        ControlFlow::Continue(visit)
    }
}

impl<Parent> Variants<Parent> for () {}

impl<Parent, Head, Tail> Variants<Parent> for (Head, Tail)
where
    Head: HasShape,
    Head: Variant<Parent = Parent>
        + ReflectStruct<Parent, Head::StructShape, <Head::Fields as Count>::Count>,
    Head::Fields: Fields<Parent> + Count,
    Tail: Variants<Parent>,
{
    const LEN: usize = 1 + Tail::LEN;

    fn for_each<V>(parent: &Parent, visit: V) -> ControlFlow<V::Break, V>
    where
        V: VariantVisitor<Parent>,
    {
        let visit = visit.visit::<Head>(parent)?;
        Tail::for_each(parent, visit)
    }
}

// ---

trait Count {
    type Count;
}
struct ZeroOrMany;
struct One;

impl Count for () {
    type Count = ZeroOrMany;
}
impl<F> Count for (F, ()) {
    type Count = One;
}
impl<A, B, C: Count> Count for (A, (B, C)) {
    type Count = ZeroOrMany;
}

// --

trait TypeKind<Shape> {
    type Kind;
}

impl<T: Struct + HasShape> TypeKind<T::Shape> for T
where
    T: StructKind<T::StructShape>,
{
    type Kind = <T as StructKind<T::StructShape>>::Kind;
}

trait StructKind<StructShape> {
    type Kind;
}
impl<T> StructKind<NamedStructShape> for T {
    type Kind = kind::Struct;
}
impl<T> StructKind<UnitStructShape> for T {
    type Kind = kind::Unit;
}
impl<T: Struct> StructKind<TupleStructShape> for T
where
    T: TupleStructKind<T::Fields>,
{
    type Kind = <T as TupleStructKind<T::Fields>>::Kind;
}

trait TupleStructKind<Fields> {
    type Kind;
}
impl<T> TupleStructKind<()> for T
where
    T: Struct,
{
    type Kind = kind::Tuple;
}
impl<T, A> TupleStructKind<(A, ())> for T
where
    T: Struct,
    A: Field,
{
    type Kind = kind::NewType;
}
impl<T, A, B, C> TupleStructKind<(A, (B, C))> for T
where
    T: Struct,
    C: Fields<T::Parent>,
{
    type Kind = kind::NewType;
}

mod kind {
    pub struct Enum;
    pub struct Tuple;
    pub struct Struct;
    pub struct NewType;
    pub struct Unit;
}

// --

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

trait ReflectStruct<Parent, StructShape, Count> {
    fn serialize<S>(parent: &Parent, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<T> ReflectType<StructShape> for T
where
    T: Struct,
    T::Fields: Count,
    T: ReflectStruct<T, T::StructShape, <T::Fields as Count>::Count>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <T as ReflectStruct<_, _, _>>::serialize(self, serializer)
    }
}

impl<Parent, T, C> ReflectStruct<Parent, NamedStructShape, C> for T
where
    T: Struct<Parent = Parent>,
    T::Fields: Fields<Parent>,
{
    fn serialize<S>(parent: &Parent, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct Visit<S>(S);
        impl<Parent, Ser: SerializeStruct> FieldVisitor<Parent> for Visit<Ser> {
            type Break = Ser::Error;

            fn visit<F>(mut self, value: &F::Type) -> ControlFlow<Self::Break, Self>
            where
                F: Field<Parent = Parent>,
                F::Type: Serialize,
            {
                match self.0.serialize_field(F::IDENT.unwrap(), value) {
                    Ok(_) => ControlFlow::Continue(self),
                    Err(err) => ControlFlow::Break(err),
                }
            }
        }

        let visit = Visit(serializer.serialize_struct(T::IDENT, T::Fields::LEN)?);
        match T::Fields::for_each(parent, visit) {
            ControlFlow::Continue(c) => c.0.end(),
            ControlFlow::Break(err) => Err(err),
        }
    }
}

impl<Parent, T, OnlyField> ReflectStruct<Parent, TupleStructShape, One> for T
where
    T: Struct<Parent = Parent, Fields = (OnlyField, ())>,
    OnlyField: Field<Parent = Parent, Type: Serialize>,
    T::Fields: Fields<Parent>,
{
    fn serialize<S>(parent: &Parent, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct(T::IDENT, OnlyField::try_get_ref(parent).unwrap())
    }
}

impl<Parent, T> ReflectStruct<Parent, TupleStructShape, ZeroOrMany> for T
where
    T: Struct<Parent = Parent>,
    T::Fields: Fields<Parent>,
{
    fn serialize<S>(parent: &Parent, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct Visit<S>(S);
        impl<Parent, Ser: SerializeTupleStruct> FieldVisitor<Parent> for Visit<Ser> {
            type Break = Ser::Error;

            fn visit<F>(mut self, value: &F::Type) -> ControlFlow<Self::Break, Self>
            where
                F: Field<Parent = Parent>,
                F::Type: Serialize,
            {
                match self.0.serialize_field(value) {
                    Ok(..) => ControlFlow::Continue(self),
                    Err(err) => ControlFlow::Break(err),
                }
            }
        }

        let visit = Visit(serializer.serialize_tuple_struct(T::IDENT, T::Fields::LEN)?);
        match T::Fields::for_each(parent, visit) {
            ControlFlow::Continue(s) => s.0.end(),
            ControlFlow::Break(err) => Err(err),
        }
    }
}

// enum

impl<T> ReflectType<EnumShape> for T
where
    T: Enum,
    T::Variants: Variants<T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct Visit<S>(S);
        impl<S: Serializer, Parent> VariantVisitor<Parent> for Visit<S> {
            type Break = Result<S::Ok, S::Error>;

            fn visit<T>(self, parent: &Parent) -> ControlFlow<Self::Break, Self>
            where
                T: ReflectStruct<Parent, T::StructShape, <T::Fields as Count>::Count>,
                T: Variant<Parent = Parent>,
                T::Fields: Fields<Parent> + Count,
            {
                if T::is_active(parent) {
                    ControlFlow::Break(<T as ReflectStruct<
                        Parent,
                        T::StructShape,
                        <T::Fields as Count>::Count,
                    >>::serialize(parent, self.0))
                } else {
                    ControlFlow::Continue(self)
                }
            }
        }

        T::Variants::for_each(self, Visit(serializer))
            .break_value()
            .expect("one variant must be active")
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
    println!(
        "{}",
        serde_json::to_string(&Reflect(&C::C(3, "hey"))).unwrap()
    );
    println!(
        "{}",
        serde_json::to_string(&Reflect(&C::D { x: "hey" })).unwrap()
    );
}
