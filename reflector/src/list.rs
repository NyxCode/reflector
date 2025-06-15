use crate::{Field, Variant};
use std::marker::Freeze;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct Cons<A, B>(pub A, pub B);

pub trait List {
    const LENGTH: usize;
}
pub trait FieldList: List {}
pub trait SizedFieldList: FieldList {
    type Types: List;
}

pub trait NamedFieldList: FieldList {
    const NAMES: &'static [&'static str];

    #[doc(hidden)]
    type NameList: Freeze + Copy + 'static;
    #[doc(hidden)]
    const NAME_LIST: Self::NameList;
}
pub trait VariantList: List {
    const NAMES: &'static [&'static str];

    #[doc(hidden)]
    type NameList: Freeze + Copy + 'static;
    #[doc(hidden)]
    const NAME_LIST: Self::NameList;
}

// impls

impl List for () {
    const LENGTH: usize = 0;
}

impl<Head, Tail> List for Cons<Head, Tail>
where
    Tail: List,
{
    const LENGTH: usize = 1 + Tail::LENGTH;
}

impl SizedFieldList for () {
    type Types = ();
}

impl<Head, Tail> SizedFieldList for Cons<Head, Tail>
where
    Head: Field<Type: Sized>,
    Tail: SizedFieldList,
{
    type Types = Cons<Head::Type, Tail::Types>;
}

impl FieldList for () {}

impl<Head, Tail> FieldList for Cons<Head, Tail>
where
    Tail: FieldList,
    Head: Field,
{
}

impl NamedFieldList for () {
    const NAMES: &'static [&'static str] = &[];
    type NameList = ();
    const NAME_LIST: Self::NameList = ();
}

impl<Head, Tail> NamedFieldList for Cons<Head, Tail>
where
    Head: Field,
    Tail: NamedFieldList,
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

impl VariantList for () {
    const NAMES: &'static [&'static str] = &[];
    type NameList = ();
    const NAME_LIST: Self::NameList = ();
}

impl<Head, Tail> VariantList for Cons<Head, Tail>
where
    Head: Variant,
    Tail: VariantList,
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
    const NAME_LIST: Self::NameList = const { Cons(Head::IDENT, Tail::NAME_LIST) };
}
