use std::ops::ControlFlow;
pub use reflector_derive::Reflect;

pub struct StructShape;
pub struct EnumShape;

pub struct NamedStructShape;
pub struct TupleStructShape;
pub struct UnitStructShape;


pub trait Type {}

pub trait HasShape {
    type Shape;
}

/// A struct type or an enum variant
pub trait Struct {
    /// Parent type of this struct. 
    /// If `Self: Type`, then `Self::Parent = Self`.
    /// If `Self: Variant`, then `Self::Parent: Enum`.
    type Parent: Type;
    /// Church list of struct fields where each element is a `Field` and possibly `Named`.
    type Fields;
    
    type StructShape;
    
    const IDENT: &'static str;
}

/// An enum type
pub trait Enum: Type {
    /// Church list of enum variants where each element is a `Variant`.
    type Variants;
    
    const IDENT: &'static str;
}

/// An enum variant
pub trait Variant: Struct<Parent: Enum> {
    const INDEX: usize;
    
    fn is_active(p: &Self::Parent) -> bool;
}

/// A field of a struct or enum variant.
pub trait Field {
    /// Type of this field
    type Type: ?Sized;
    /// Parent type in which this field is contained.
    /// Can be either a `Struct` or `Enum`.
    type Parent: Type;
    
    const IDENT: Option<&'static str>;
    const INDEX: usize;

    /// Obtains a reference to the value of this field.
    /// Returns `None` iff `Self::Parent: Enum` and this field is contained in a variant which
    /// is not currently active.
    fn try_get_ref(p: &Self::Parent) -> Option<&Self::Type>;
    /// Obtains a mutable reference to the value of this field.
    /// Returns `None` iff `Self::Parent: Enum` and this field is contained in a variant which
    /// is not currently active.
    fn try_get_mut(p: &mut Self::Parent) -> Option<&mut Self::Type>;
}

#[doc(hidden)]
pub trait HasField<F> {
    type Type;
}

