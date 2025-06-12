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

pub trait HasName {
    const NAME: &'static str;
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
    fn is_active(p: &Self::Parent) -> bool;
}

/// A field of a struct or enum variant.
pub trait Field {
    /// Type of this field
    type Type: ?Sized;
    /// Parent type in which this field is contained.
    /// Can be either a `Struct` or `Enum`.
    type Parent: Type;
    /// Identifier of this field. Either `&'static str` or `usize`.
    type Ident;
    
    const IDENT: Self::Ident;

    /// Obtains a reference to the value of this field.
    /// Returns `None` iff `Self::Parent: Enum` and this field is contained in a variant which
    /// is not currently active.
    fn try_get_ref(p: &Self::Parent) -> Option<&Self::Type>;
    /// Obtains a mutable reference to the value of this field.
    /// Returns `None` iff `Self::Parent: Enum` and this field is contained in a variant which
    /// is not currently active.
    fn try_get_mut(p: &mut Self::Parent) -> Option<&mut Self::Type>;
}


pub trait NamedField: Field<Ident = &'static str> {}
pub trait TupleField: Field<Ident = usize> {}

impl<F: Field<Ident = &'static str>> NamedField for F {}
impl<F: Field<Ident = usize>> TupleField for F {}

#[doc(hidden)]
pub trait HasField<F> {
    type Type;
}

// ...
