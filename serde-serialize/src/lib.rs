mod visit;

use std::error::Error;
use reflector::*;
use serde::ser::{
    SerializeStruct, SerializeStructVariant, SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};
use visit::{FieldVisitor, Fields, VariantVisitor, Variants};

pub struct Reflect<'a, T>(&'a T);

impl<'a, T> Serialize for Reflect<'a, T>
where
    T: Introspect<Root = T> + Impl<T::Root, T::Kind>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        T::serialize(self.0, serializer)
    }
}

trait Impl<Root, Kind> {
    fn serialize<S: Serializer>(root: &Root, s: S) -> Result<S::Ok, S::Error>;
}

trait ImplStruct<Root, RootKind, Shape> {
    fn serialize<S: Serializer>(root: &Root, s: S) -> Result<S::Ok, S::Error>;
}

trait ImplTuple<Root, RootKind, Fields> {
    fn serialize<S: Serializer>(root: &Root, s: S) -> Result<S::Ok, S::Error>;
}


impl<I: Struct> Impl<I::Root, StructType> for I
where
    I: ImplStruct<I::Root, <I::Root as Introspect>::Kind, I::Shape>,
{
    fn serialize<S: Serializer>(root: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        <I as ImplStruct<_, _, _>>::serialize(root, s)
    }
}

// struct I;
impl<I: Struct> ImplStruct<I::Root, StructType, UnitStruct> for I {
    fn serialize<S: Serializer>(_: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_unit_struct(I::IDENT)
    }
}
// enum Root { I, .. };
impl<I: Variant> ImplStruct<I::Root, EnumType, UnitStruct> for I {
    fn serialize<S: Serializer>(_: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_unit_variant(I::Root::IDENT, I::INDEX, I::IDENT)
    }
}

// struct I { .. }
impl<I: Struct> ImplStruct<I::Root, StructType, NamedStruct> for I
where
    I::Fields: Fields<I::Root>,
{
    fn serialize<S: Serializer>(root: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        struct Visit<S>(S);
        impl<Root, S: SerializeStruct> FieldVisitor<Root> for Visit<S> {
            type Error = S::Error;

            fn visit<F>(mut self, value: &F::Type) -> Result<Self, Self::Error>
            where
                F: Field<Root = Root, Type: Serialize>,
            {
                self.0.serialize_field(F::IDENT.unwrap(), value)?;
                Ok(self)
            }
        }

        let visit = Visit(s.serialize_struct(I::IDENT, I::Fields::LEN)?);
        I::Fields::for_each(root, visit)?.0.end()
    }
}
// enum Parent { I { .. }, .. } }
impl<I: Variant> ImplStruct<I::Root, EnumType, NamedStruct> for I
where
    I::Fields: Fields<I::Root>,
{
    fn serialize<S: Serializer>(root: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        struct Visit<S>(S);
        impl<Root, S: SerializeStructVariant> FieldVisitor<Root> for Visit<S> {
            type Error = S::Error;

            fn visit<F>(mut self, value: &F::Type) -> Result<Self, Self::Error>
            where
                F: Field<Root = Root, Type: Serialize>,
            {
                self.0.serialize_field(F::IDENT.unwrap(), value)?;
                Ok(self)
            }
        }

        let visit = Visit(s.serialize_struct_variant(
            I::Root::IDENT,
            I::INDEX,
            I::IDENT,
            I::Fields::LEN,
        )?);
        I::Fields::for_each(root, visit)?.0.end()
    }
}

// enum I { .. }
impl<I: Enum> Impl<I::Root, EnumType> for I
where
    I::Variants: Variants<I::Root>,
{
    fn serialize<S: Serializer>(root: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        struct Visit<S>(S);
        impl<Root, S: Serializer> VariantVisitor<Root> for Visit<S> {
            type Error = Result<S::Ok, S::Error>;

            fn visit<T>(self, root: &Root) -> Result<Self, Self::Error>
            where
                T: Variant<Root = Root, Fields: Fields<Root>> + Impl<Root, T::Kind>,
            {
                if T::is_active(root) {
                    Err(T::serialize(root, self.0))
                } else {
                    Ok(self)
                }
            }
        }

        I::Variants::for_each(root, Visit(s)).err().unwrap()
    }
}

// struct I(..);
impl<I: Struct> ImplStruct<I::Root, <I::Root as Introspect>::Kind, TupleStruct> for I
where
    I: ImplTuple<I::Root, <I::Root as Introspect>::Kind, I::Fields>,
{
    fn serialize<S: Serializer>(root: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        <I as ImplTuple<_, _, _>>::serialize(root, s)
    }
}

// struct I();
impl<I: Struct> ImplTuple<I::Root, StructType, ()> for I {
    fn serialize<S: Serializer>(_: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_tuple_struct(I::IDENT, 0)?.end()
    }
}
// enum Root {  I(), .. }
impl<I: Variant> ImplTuple<I::Root, EnumType, ()> for I {
    fn serialize<S: Serializer>(_: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_tuple_variant(I::Root::IDENT, I::INDEX, I::IDENT, 0)?
            .end()
    }
}

// struct I(A);
impl<I: Struct, A: Field<Root = I::Root, Type: Serialize>> ImplTuple<I::Root, StructType, (A, ())>
    for I
{
    fn serialize<S: Serializer>(root: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_newtype_struct(I::IDENT, A::try_get_ref(root).unwrap())
    }
}
// enum Root { I(A), .. };
impl<I: Variant, A: Field<Root = I::Root, Type: Serialize>> ImplTuple<I::Root, EnumType, (A, ())>
    for I
{
    fn serialize<S: Serializer>(root: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_newtype_variant(
            I::Root::IDENT,
            I::INDEX,
            I::IDENT,
            A::try_get_ref(root).unwrap(),
        )
    }
}

// struct I(A, B, ..);
impl<I: Struct, A, B, C> ImplTuple<I::Root, StructType, (A, (B, C))> for I
where
    I::Fields: Fields<I::Root>,
{
    fn serialize<S: Serializer>(root: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        struct Visit<S>(S);

        impl<Root, S: SerializeTupleStruct> FieldVisitor<Root> for Visit<S> {
            type Error = S::Error;

            fn visit<F>(mut self, value: &F::Type) -> Result<Self, Self::Error>
            where
                F: Field<Root = Root, Type: Serialize>,
            {
                self.0.serialize_field(value)?;
                Ok(self)
            }
        }

        let visit = Visit(s.serialize_tuple_struct(I::IDENT, I::Fields::LEN)?);
        I::Fields::for_each(root, visit)?.0.end()
    }
}
// enum Root { I(A, B, ..), .. };
impl<I: Variant, A, B, C> ImplTuple<I::Root, EnumType, (A, (B, C))> for I
where
    I::Fields: Fields<I::Root>,
{
    fn serialize<S: Serializer>(root: &I::Root, s: S) -> Result<S::Ok, S::Error> {
        struct Visit<S>(S);

        impl<Root, S: SerializeTupleVariant> FieldVisitor<Root> for Visit<S> {
            type Error = S::Error;

            fn visit<F>(mut self, value: &F::Type) -> Result<Self, Self::Error>
            where
                F: Field<Root = Root, Type: Serialize>,
            {
                self.0.serialize_field(value)?;
                Ok(self)
            }
        }

        let visit =
            Visit(s.serialize_tuple_variant(I::Root::IDENT, I::INDEX, I::IDENT, I::Fields::LEN)?);
        I::Fields::for_each(root, visit)?.0.end()
    }
}

#[test]
fn works() {
    use serde_json::to_string;

    #[derive(Introspect)]
    struct A<X> {
        a: i32,
        b: X,
    }

    println!("{}", to_string(&Reflect(&A { a: 42, b: 3u8 })).unwrap());
    println!("{}", to_string(&Reflect(&A { a: 42, b: "hey" })).unwrap());

    #[derive(Introspect)]
    struct B<'a>(i32, &'a str);
    println!("{}", to_string(&Reflect(&B(42, "hey"))).unwrap());

    #[derive(Introspect)]
    enum C<'a> {
        A,
        B(i32),
        C(i32, &'a str),
        D { x: &'a str },
    };
    println!("{}", to_string(&Reflect(&C::A)).unwrap());
    println!("{}", to_string(&Reflect(&C::B(3))).unwrap());
    println!("{}", to_string(&Reflect(&C::C(3, "hey"))).unwrap());
    println!("{}", to_string(&Reflect(&C::D { x: "hey" })).unwrap());
}
