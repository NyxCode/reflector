use crate::{ImplKind, Impl};
use reflector::{Cons, Enum, Field, Introspect, Struct, Variant};
use serde::Serialize;

pub trait FieldVisitor<Root>: Sized {
    type Error;

    fn visit<F>(self, value: &F::Type) -> Result<Self, Self::Error>
    where
        F: Field<Root = Root, Type: Serialize>;
}

pub trait Fields<Root> {
    const LEN: usize = 0;

    fn for_each<V: FieldVisitor<Root>>(_root: &Root, visit: V) -> Result<V, V::Error> {
        Ok(visit)
    }
}

impl<Root> Fields<Root> for () {}

impl<Root, Head, Tail> Fields<Root> for Cons<Head, Tail>
where
    Head: Field<Root = Root, Type: Serialize>,
    Tail: Fields<Root>,
{
    const LEN: usize = 1 + Tail::LEN;

    fn for_each<V: FieldVisitor<Root>>(root: &Root, visit: V) -> Result<V, V::Error> {
        let visit = visit.visit::<Head>(Head::try_get_ref(root).unwrap())?;
        Tail::for_each(root, visit)
    }
}

pub trait VariantVisitor<Root>: Sized {
    type Error;

    fn visit<T>(self, root: &Root) -> Result<Self, Self::Error>
    where
        T: Variant<Root = Root, Fields: Fields<Root>> + Impl;
}

pub trait Variants<Root> {
    const LEN: usize = 0;

    fn for_each<V: VariantVisitor<Root>>(_: &Root, visit: V) -> Result<V, V::Error> {
        Ok(visit)
    }
}

impl<Root> Variants<Root> for () {}

impl<Root, Head, Tail> Variants<Root> for Cons<Head, Tail>
where
    Head: Variant<Root = Root, Fields: Fields<Root>> + Impl,
    Tail: Variants<Root>,
{
    const LEN: usize = 1 + Tail::LEN;

    fn for_each<V>(parent: &Root, visit: V) -> Result<V, V::Error>
    where
        V: VariantVisitor<Root>,
    {
        let visit = visit.visit::<Head>(parent)?;
        Tail::for_each(parent, visit)
    }
}
