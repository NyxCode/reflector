#![feature(freeze)]

pub use reflector_derive::Introspect;
use std::marker::Freeze;

pub trait Introspect {
    const IDENT: &'static str;

    type Root: Introspect;
    type Kind: Kind;
}

pub trait Struct: Introspect {
    type Shape: StructShape;
    type Fields: FieldList;
}

pub trait NamedStruct: Struct<Shape = NamedShape, Fields: NamedFields> {}
impl<S> NamedStruct for S where S: Struct<Shape = NamedShape, Fields: NamedFields> {}

pub trait TupleStruct: Struct<Shape = TupleShape> {}
impl<S> TupleStruct for S where S: Struct<Shape = TupleShape> {}

pub trait UnitStruct: Struct<Shape = UnitShape> {}
impl<S> UnitStruct for S where S: Struct<Shape = UnitShape> {}

pub trait SizedStruct: Struct<Fields: SizedFields> {
    type FieldTypes;

    fn from_values(values: Self::FieldTypes) -> Self;
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
pub trait StructShape {}
pub struct UnitShape;
pub struct TupleShape;
pub struct NamedShape;
impl StructShape for UnitShape {}
impl StructShape for TupleShape {}
impl StructShape for NamedShape {}

// kinds
pub trait Kind {}
pub struct StructKind;
pub struct EnumKind;
impl Kind for StructKind {}
impl Kind for EnumKind {}

// lists
pub trait List {
    const LENGTH: usize;
}
impl List for () {
    const LENGTH: usize = 0;
}
impl<Head, Tail> List for Cons<Head, Tail>
where
    Tail: List,
{
    const LENGTH: usize = 1 + Tail::LENGTH;
}

pub trait SizedFields {
    type Types: List;
}
impl SizedFields for () {
    type Types = ();
}
impl<Head, Tail> SizedFields for Cons<Head, Tail>
where
    Head: Field<Type: Sized>,
    Tail: SizedFields,
{
    type Types = Cons<Head::Type, Tail::Types>;
}

pub trait FieldList {}
impl FieldList for () {}
impl<Head, Tail> FieldList for Cons<Head, Tail>
where
    Tail: FieldList,
    Head: Field,
{
}

pub trait NamedFields: List {
    const NAMES: &'static [&'static str];
    #[doc(hidden)]
    type NameList: Freeze + Copy + 'static;
    #[doc(hidden)]
    const NAME_LIST: Self::NameList;
}
impl NamedFields for () {
    const NAMES: &'static [&'static str] = &[];
    type NameList = ();
    const NAME_LIST: Self::NameList = ();
}
impl<Head, Tail> NamedFields for Cons<Head, Tail>
where
    Head: Field,
    Tail: NamedFields,
{
    const NAMES: &'static [&'static str] = unsafe {
        let name_list: &'static Self::NameList = &Self::NAME_LIST;
        // SAFETY:  `Self::Idents` is `Cons<&'static str, Cons<.., ()>>`, only containing
        //          `&'static str`s. Since `Cons` is `#[repr(C)]`, the layout of `name_list` is
        //          the same as `[&str]`.
        std::slice::from_raw_parts(
            name_list as *const Self::NameList as *const &str,
            Self::LENGTH,
        )
    };
    type NameList = Cons<&'static str, Tail::NameList>;
    const NAME_LIST: Self::NameList = const { Cons(Head::IDENT.unwrap(), Tail::NAME_LIST) };
}

macro_rules! map_types {
    () => {
        ()
    };
}

#[doc(hidden)]
pub trait HasField<F> {
    type Type;
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct Cons<A, B>(pub A, pub B);
