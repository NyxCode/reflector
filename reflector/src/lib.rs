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

