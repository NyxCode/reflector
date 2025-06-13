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

/*
pub trait List {
    const LEN: usize;
}

impl List for () {
    const LEN: usize = 0;
}

impl<Head, Tail> List for (Head, Tail)
where
    Tail: List,
{
    const LEN: usize = 1 + Tail::LEN;
}

pub trait FieldList: List {}

impl FieldList for () {}
impl<Head, Tail> FieldList for (Head, Tail)
where
    Head: Field,
    Tail: FieldList,
{
}
*/
pub type ValuesOf<Fields> = <Fields as FieldValues>::Values;

pub trait FieldValues {
    type Values;
}

impl FieldValues for () {
    type Values = ();
}

impl<Head, Tail> FieldValues for (Head, Tail)
where
    Head: Field<Type: Sized>,
    Tail: FieldValues,
{
    type Values = (Head::Type, Tail::Values);
}


#[repr(C)] struct Cons<A, B>(A, B);

trait _AsCons {
    type AsCons;
}




trait Homogenous<T> {
    const LEN: usize;
    type AsCons;
    
    fn into_cons(self) -> Self::AsCons;
    fn as_slice(&self) -> &[T];
}

impl<T> Homogenous<T> for () {
    const LEN: usize = 0;
    type AsCons = ();

    fn into_cons(self) -> Self::AsCons {
        ()
    }

    fn as_slice(&self) -> &[T] {
        &[]
    }
}

impl<Head, Tail> Homogenous<Head> for (Head, Tail) where Tail: Homogenous<Tail> {
    const LEN: usize = 1 + Tail::LEN;
    type AsCons = Cons<Head, Tail::AsCons>;

    fn into_cons(self) -> Self::AsCons {
        Cons(self.0, self.1.into_cons())
    }

    fn as_slice(&self) -> &[Head] {
        unimplemented!()
    }
}
