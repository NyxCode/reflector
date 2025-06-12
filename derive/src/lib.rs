use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{
    Error, Field, Fields, Generics, Ident, Index, Item, ItemEnum, ItemStruct, Member, Result,
    Variant, Visibility,
};

#[proc_macro_derive(Reflect)]
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

    println!("{output}");

    Ok(quote! {
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
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let field_struct_idents = (0..fields.len())
        .map(|i| format_ident!("{struct_ident}_{i}"))
        .collect::<Vec<Ident>>();

    let field_list = church_list(
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
                i,
                field,
                variant,
            )
        })
        .collect::<TokenStream>();
    let type_impl = (parent_ident == struct_ident).then(|| expand_type(generics, parent_ident));
    let struct_shape = match fields {
        Fields::Named(_) => quote!(NamedStructShape),
        Fields::Unnamed(_) => quote!(TupleStructShape),
        Fields::Unit => quote!(UnitStructShape),
    };

    quote! {
        #field_items
        #type_impl

        impl #impl_generics ::reflector::Struct for #struct_ident #type_generics {
            type Parent = #parent_ident #type_generics;
            type Fields = #field_list;
            type StructShape = ::reflector::#struct_shape;

            const IDENT: &'static str = stringify!(#name);
        }
        impl #impl_generics ::reflector::HasShape for #struct_ident #type_generics {
            type Shape = ::reflector::StructShape;
        }
    }
}

fn for_enum(parent: &ItemEnum) -> TokenStream {
    let parent_ident = &parent.ident;
    let (impl_generics, type_generics, where_clause) = parent.generics.split_for_impl();

    let variant_struct_idents = parent
        .variants
        .iter()
        .map(|v| format_ident!("{}_{}", parent.ident, v.ident))
        .collect::<Vec<Ident>>();

    let variant_list = church_list(
        variant_struct_idents
            .iter()
            .map(|i| quote!(#i #type_generics)),
    );

    let variants = (0..parent.variants.len())
        .map(|i| for_variant(parent, &parent.variants[i], &variant_struct_idents[i]));

    let type_impl = expand_type(&parent.generics, &parent.ident);

    quote! {
        #(#variants)*

        #type_impl
        impl #impl_generics ::reflector::Enum for #parent_ident #type_generics {
            type Variants = #variant_list;

            const IDENT: &'static str = stringify!(#parent_ident);
        }

        impl #impl_generics ::reflector::HasShape for #parent_ident #type_generics {
            type Shape = ::reflector::EnumShape;
        }

    }
}

fn for_variant(parent: &ItemEnum, variant: &Variant, variant_struct_ident: &Ident) -> TokenStream {
    let generics = &parent.generics;
    let parent_ident = &parent.ident;
    let vis = &parent.vis;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let is_active = {
        let variant_ident = &variant.ident;
        let pattern = match variant.fields {
            Fields::Named(_) => quote!({ .. }),
            Fields::Unnamed(_) => quote!((..)),
            Fields::Unit => quote!(),
        };
        quote!(matches!(p, Self::Parent::#variant_ident #pattern))
    };

    let struct_items = expand_struct(
        &parent.vis,
        &parent.ident,
        variant_struct_ident,
        &variant.ident,
        &parent.generics,
        &variant.fields,
        Some(&variant),
    );
    quote! {
        #vis struct #variant_struct_ident #generics (#parent_ident #generics);
        impl #impl_generics ::reflector::Variant for #variant_struct_ident #type_generics {
            fn is_active(p: &Self::Parent) -> bool { #is_active }
        }
        #struct_items
    }
}

fn church_list(elements: impl DoubleEndedIterator<Item = TokenStream>) -> TokenStream {
    elements
        .rev()
        .fold(quote![()], |list, element| quote![(#element, #list)])
}

fn expand_type(generics: &Generics, ident: &Ident) -> TokenStream {
    let (impl_generics, type_generics, _) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::reflector::Type for #ident #type_generics {}
    }
}

fn generate_field_items(
    parent_vis: &Visibility,
    parent_ident: &Ident,
    parent_generics: &Generics,
    field_struct_ident: &Ident,
    field_idx: usize,
    field: &Field,
    inside_variant: Option<&Variant>,
) -> TokenStream {
    let field_type = &field.ty;
    let (impl_generics, type_generics, where_clause) = parent_generics.split_for_impl();

    let accessor = accessor(
        inside_variant,
        &match &field.ident {
            None => Member::Unnamed(Index::from(field_idx)),
            Some(ident) => Member::Named(ident.clone()),
        },
    );

    let ident = match &field.ident {
        None => quote! {
            type Ident = usize;
            const IDENT: Self::Ident = #field_idx;
        },
        Some(ident) => quote! {
            type Ident = &'static str;
            const IDENT: Self::Ident = stringify!(#ident);
        },
    };

    quote! {
        #[allow(dead_code)]
        #parent_vis struct #field_struct_ident #parent_generics(#parent_ident #type_generics);
        impl #impl_generics ::reflector::HasField<#field_struct_ident #type_generics> for #parent_ident #type_generics {
            type Type = #field_type;
        }
        impl #impl_generics ::reflector::Field for #field_struct_ident #type_generics {
            type Type = <#parent_ident #type_generics as ::reflector::HasField<Self>>::Type;
            type Parent = #parent_ident #type_generics;

            #ident

            fn try_get_ref(p: &Self::Parent) -> Option<&Self::Type> { #accessor }
            fn try_get_mut(p: &mut Self::Parent) -> Option<&mut Self::Type> { #accessor }
        }
    }
}

fn accessor(inside_variant: Option<&Variant>, field: &Member) -> TokenStream {
    let inside_variant = inside_variant.iter().map(|v| &v.ident);

    quote! {
        match p {
            Self::Parent #(:: #inside_variant)* { #field: x, .. } => Some(x),
            _ => None,
        }
    }
}
