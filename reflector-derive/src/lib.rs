use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{
    Error, Field, Fields, Generics, Ident, Index, Item, ItemEnum, ItemStruct, Member, Result,
    Variant, Visibility,
};

#[proc_macro_derive(Introspect)]
pub fn my_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    entry(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn entry(input: proc_macro::TokenStream) -> Result<TokenStream> {
    let input = syn::parse::<Item>(input)?;
    let output = match &input {
        Item::Struct(s) => for_struct(s),
        Item::Enum(e) => for_enum(e),
        x => return Err(Error::new(x.span(), "unsupported item")),
    };

    Ok(quote! {
        #[allow(dead_code, non_camel_case_types)]
        const _: () = { #output };
    })
}

fn for_struct(s: &ItemStruct) -> TokenStream {
    expand_struct(
        &s.vis,
        &s.ident,
        &s.ident,
        &s.ident,
        &s.generics,
        &s.fields,
        None,
    )
}

fn expand_struct(
    parent_vis: &Visibility,
    parent_ident: &Ident,
    struct_ident: &Ident,
    name: &Ident,
    generics: &Generics,
    fields: &Fields,
    variant: Option<&Variant>,
) -> TokenStream {
    let (impl_generics, type_generics, ..) = generics.split_for_impl();

    let field_struct_idents = (0..fields.len())
        .map(|i| format_ident!("{struct_ident}_{i}"))
        .collect::<Vec<Ident>>();

    let field_list = type_list(
        field_struct_idents
            .iter()
            .map(|id| quote!(#id #type_generics)),
    );

    let field_items = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            generate_field_items(
                parent_vis,
                parent_ident,
                generics,
                &field_struct_idents[i],
                i as u32,
                field,
                variant,
            )
        })
        .collect::<TokenStream>();
    let shape = match fields {
        Fields::Named(_) => quote!(NamedShape),
        Fields::Unnamed(_) => quote!(TupleShape),
        Fields::Unit => quote!(UnitShape),
    };

    let sized = {
        let args: Vec<_> = (0..fields.len()).map(|i| format_ident!("x{i}")).collect();
        let values = args
            .iter()
            .rev()
            .fold(quote!(()), |acc, f| quote![::reflector::Cons(#f, #acc)]);

        let fields = fields
            .members()
            .zip(&args)
            .map(|(member, arg)| quote!(#member: #arg));
        let variant = variant.map(|Variant { ident, .. }| quote!(:: #ident));
        quote! {
            impl #impl_generics ::reflector::SizedStruct for #struct_ident #type_generics {
                type FieldTypes = <Self::Fields as ::reflector::SizedFieldList>::Types;

                fn from_values(#values: Self::FieldTypes) -> Self::Root {
                    Self::Root #variant { #(#fields),* }
                }
            }
        }
    };

    quote! {
        #field_items

        impl #impl_generics ::reflector::Struct for #struct_ident #type_generics {
            type Fields = #field_list;
            type Shape = ::reflector::#shape;
        }

        #sized

        impl #impl_generics ::reflector::Introspect for #struct_ident #type_generics {
            const IDENT: &'static str = stringify!(#name);

            type Root = #parent_ident #type_generics;
            type Kind = ::reflector::StructKind;
        }
    }
}

fn for_enum(parent: &ItemEnum) -> TokenStream {
    let parent_ident = &parent.ident;
    let (impl_generics, type_generics, ..) = parent.generics.split_for_impl();

    let variant_struct_idents = parent
        .variants
        .iter()
        .map(|v| format_ident!("{}_{}", parent.ident, v.ident))
        .collect::<Vec<Ident>>();

    let variant_list = type_list(
        variant_struct_idents
            .iter()
            .map(|i| quote!(#i #type_generics)),
    );

    let variants = (0..parent.variants.len()).map(|i| {
        for_variant(
            parent,
            &parent.variants[i],
            i as u32,
            &variant_struct_idents[i],
        )
    });

    quote! {
        #(#variants)*

        impl #impl_generics ::reflector::Enum for #parent_ident #type_generics {
            type Variants = #variant_list;

        }

        impl #impl_generics ::reflector::Introspect for #parent_ident #type_generics {
            const IDENT: &'static str = stringify!(#parent_ident);

            type Root = #parent_ident #type_generics;
            type Kind = ::reflector::EnumKind;
        }
    }
}

fn for_variant(
    parent: &ItemEnum,
    variant: &Variant,
    index: u32,
    variant_struct_ident: &Ident,
) -> TokenStream {
    let generics = &parent.generics;
    let parent_ident = &parent.ident;
    let vis = &parent.vis;
    let (impl_generics, type_generics, ..) = generics.split_for_impl();

    let is_active = {
        let variant_ident = &variant.ident;
        let pattern = match variant.fields {
            Fields::Named(_) => quote!({ .. }),
            Fields::Unnamed(_) => quote!((..)),
            Fields::Unit => quote!(),
        };
        quote!(matches!(p, Self::Root::#variant_ident #pattern))
    };

    let struct_items = expand_struct(
        &parent.vis,
        &parent.ident,
        variant_struct_ident,
        &variant.ident,
        &parent.generics,
        &variant.fields,
        Some(variant),
    );
    quote! {
        #vis struct #variant_struct_ident #generics (#parent_ident #generics);
        impl #impl_generics ::reflector::Variant for #variant_struct_ident #type_generics {
            const INDEX: u32 = #index;

            fn is_active(p: &Self::Root) -> bool { #is_active }
        }
        #struct_items
    }
}

fn type_list(elements: impl DoubleEndedIterator<Item = TokenStream>) -> TokenStream {
    elements.rev().fold(
        quote![()],
        |list, element| quote![::reflector::Cons<#element, #list>],
    )
}

fn generate_field_items(
    parent_vis: &Visibility,
    parent_ident: &Ident,
    parent_generics: &Generics,
    field_struct_ident: &Ident,
    field_idx: u32,
    field: &Field,
    inside_variant: Option<&Variant>,
) -> TokenStream {
    let field_type = &field.ty;
    let (impl_generics, type_generics, ..) = parent_generics.split_for_impl();

    let accessor = accessor(
        inside_variant,
        &match &field.ident {
            None => Member::Unnamed(Index::from(field_idx as usize)),
            Some(ident) => Member::Named(ident.clone()),
        },
    );

    let ident = match &field.ident {
        None => quote!(None),
        Some(ident) => quote!(Some(stringify!(#ident))),
    };

    quote! {
        #parent_vis struct #field_struct_ident #parent_generics(#parent_ident #type_generics);
        impl #impl_generics ::reflector::HasField<#field_struct_ident #type_generics> for #parent_ident #type_generics {
            type Type = #field_type;
        }
        impl #impl_generics ::reflector::Field for #field_struct_ident #type_generics {
            type Type = <#parent_ident #type_generics as ::reflector::HasField<Self>>::Type;
            type Root = #parent_ident #type_generics;

            const IDENT: Option<&'static str> = #ident;
            const INDEX: u32 = #field_idx;

            fn try_get_ref(p: &Self::Root) -> Option<&Self::Type> { #accessor }
            fn try_get_mut(p: &mut Self::Root) -> Option<&mut Self::Type> { #accessor }
        }
    }
}

fn accessor(inside_variant: Option<&Variant>, field: &Member) -> TokenStream {
    let inside_variant = inside_variant.iter().map(|v| &v.ident);

    quote! {
        match p {
            Self::Root #(:: #inside_variant)* { #field: x, .. } => Some(x),
            _ => None,
        }
    }
}
