#![feature(freeze)]

use std::marker::Freeze;
pub use reflector_derive::Introspect;

pub trait Introspect {
    const IDENT: &'static str;

    type Root: Introspect;
    type Kind;
}

pub trait Struct: Introspect {
    type Shape;
    type Fields;
}

pub trait FromValues: Struct<Fields: FieldValues> {
    fn from_values(values: ValuesOf<Self::Fields>) -> Self;
}

pub trait Enum: Introspect {
    type Variants;
}

pub trait Variant: Struct {
    const INDEX: u32;

    fn is_active(p: &Self::Root) -> bool;
}

pub trait Field {
    type Type: ?Sized;
    type Root: Introspect;

    const IDENT: Option<&'static str>;
    const INDEX: u32;

    fn try_get_ref(p: &Self::Root) -> Option<&Self::Type>;
    fn try_get_mut(p: &mut Self::Root) -> Option<&mut Self::Type>;
}

// struct shapes
pub struct UnitStruct;
pub struct TupleStruct;
pub struct NamedStruct;

// kinds
pub struct StructType;
pub struct EnumType;

#[doc(hidden)]
pub trait HasField<F> {
    type Type;
}






pub type ValuesOf<Fields> = <Fields as FieldValues>::Values;

pub trait FieldValues {
    type Values;
}

impl FieldValues for () {
    type Values = ();
}

impl<Head, Tail> FieldValues for Cons<Head, Tail>
where
    Head: Field<Type: Sized>,
    Tail: FieldValues,
{
    type Values = Cons<Head::Type, Tail::Values>;
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct Cons<A, B>(pub A, pub B);

pub trait Homogenous<T> {
    const LEN: usize;
    type Map<R>;

    fn map<R, F: FnMut(T) -> R>(self, f: F) -> Self::Map<R>;
    fn as_slice(&self) -> &[T];
}

impl<T> Homogenous<T> for () {
    const LEN: usize = 0;
    type Map<R> = ();

    fn map<R, F: FnMut(T) -> R>(self, _: F) -> Self::Map<R> {
        ()
    }

    fn as_slice(&self) -> &[T] {
        &[]
    }
}

impl<Head, Tail> Homogenous<Head> for Cons<Head, Tail>
where
    Tail: Homogenous<Head>,
{
    const LEN: usize = 1 + Tail::LEN;
    type Map<R> = Cons<R, Tail::Map<R>>;

    fn map<R, F: FnMut(Head) -> R>(self, mut f: F) -> Self::Map<R> {
        Cons(f(self.0), self.1.map(f))
    }

    fn as_slice(&self) -> &[Head] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const _, Self::LEN) }
    }
}

const fn as_slice<T, H: Homogenous<T>>(list: &H) -> &[T] {
    unsafe { std::slice::from_raw_parts(list as *const H as *const _, H::LEN) }
}

pub trait FieldIdents {
    type Idents: Homogenous<&'static str> + 'static + Copy + Freeze;
    const IDENTS: Self::Idents;
    const AS_REF: &'static Self::Idents;
}

impl FieldIdents for () {
    type Idents = ();
    const IDENTS: Self::Idents = ();
    const AS_REF: &'static Self::Idents = &();
}
impl<Head, Tail> FieldIdents for Cons<Head, Tail>
where
    Head: Field,
    Tail: FieldIdents,
{
    type Idents = Cons<&'static str, Tail::Idents>;
    const IDENTS: Self::Idents = const { Cons(Head::IDENT.unwrap(), Tail::IDENTS) };
    const AS_REF: &'static Self::Idents = &Self::IDENTS;
}

const A: usize = 0;

fn x() -> &'static usize {
    let z: &'static usize = &A;
    z
}

fn field_idents<T>() -> &'static [&'static str]
where
    T: Struct,
    T::Fields: FieldIdents,
{
    <T::Fields as FieldIdents>::AS_REF.as_slice()
}
