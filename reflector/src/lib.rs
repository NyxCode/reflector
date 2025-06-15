#![feature(freeze)]

mod list;

pub use list::*;
pub use reflector_derive::Introspect;

/// Anything which can be introspected - structs, enums, and enum variants, at the moment.
pub trait Introspect {
    const IDENT: &'static str;

    /// "Root" type. Refers to `Self` for every type, and to its enum for variants..
    type Root: Introspect;
    type Kind: Kind;
}

pub trait Struct: Introspect {
    type Shape: StructShape;
    type Fields: FieldList;
}

pub trait SizedStruct: Struct<Fields: SizedFieldList> {
    type FieldTypes;

    fn from_values(values: Self::FieldTypes) -> Self::Root;
}

pub trait Field {
    type Type: ?Sized;
    type Root: Introspect;

    const IDENT: Option<&'static str>;
    const INDEX: u32;

    fn try_get_ref(p: &Self::Root) -> Option<&Self::Type>;
    fn try_get_mut(p: &mut Self::Root) -> Option<&mut Self::Type>;
}

pub trait Enum: Introspect {
    type Variants: VariantList;
}

pub trait Variant: Struct {
    const INDEX: u32;

    fn is_active(p: &Self::Root) -> bool;
}

pub trait NamedStruct: Struct<Shape = NamedShape, Fields: NamedFieldList> {}
impl<S> NamedStruct for S where S: Struct<Shape = NamedShape, Fields: NamedFieldList> {}

pub trait TupleStruct: Struct<Shape = TupleShape> {}
impl<S> TupleStruct for S where S: Struct<Shape = TupleShape> {}

pub trait UnitStruct: Struct<Shape = UnitShape> {}
impl<S> UnitStruct for S where S: Struct<Shape = UnitShape> {}

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

#[doc(hidden)]
pub trait HasField<F> {
    type Type;
}
