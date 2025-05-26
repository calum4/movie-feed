use proc_macro2::Ident;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

pub(crate) fn is_credit(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    fn propagate_trait_method(data: &Data, ident: &Ident, method: Ident) -> TokenStream {
        let variant_idents = match data {
            Data::Struct(_) => unimplemented!(),
            Data::Enum(data) => data.variants.iter().map(|variant| variant.ident.clone()),
            Data::Union(_) => unimplemented!(),
        };

        quote! {
            match self {
                #(#ident::#variant_idents(credit) => credit.#method(),)*
            }
        }
    }

    let id = propagate_trait_method(&input.data, &ident, Ident::new("id", Span::call_site()));
    let title = propagate_trait_method(&input.data, &ident, Ident::new("title", Span::call_site()));
    let original_title = propagate_trait_method(
        &input.data,
        &ident,
        Ident::new("original_title", Span::call_site()),
    );
    let genres =
        propagate_trait_method(&input.data, &ident, Ident::new("genres", Span::call_site()));
    let release_date = propagate_trait_method(
        &input.data,
        &ident,
        Ident::new("release_date", Span::call_site()),
    );
    let original_language = propagate_trait_method(
        &input.data,
        &ident,
        Ident::new("original_language", Span::call_site()),
    );
    let overview = propagate_trait_method(
        &input.data,
        &ident,
        Ident::new("overview", Span::call_site()),
    );
    let credit_id = propagate_trait_method(
        &input.data,
        &ident,
        Ident::new("credit_id", Span::call_site()),
    );
    let media_type = propagate_trait_method(
        &input.data,
        &ident,
        Ident::new("media_type", Span::call_site()),
    );

    let expanded = quote! {
        impl #impl_generics crate::models::v3::credit::IsCredit for #ident #ty_generics #where_clause {
            #[inline]
            fn id(&self) -> usize {
                #id
            }

            #[inline]
            fn title(&self) -> &str {
                #title
            }

            #[inline]
            fn original_title(&self) -> &str {
                #original_title
            }

            #[inline]
            fn genres(&self) -> Vec<&dyn crate::models::v3::genres::Genre> {
                #genres
            }

            #[inline]
            fn release_date(&self) -> Option<&chrono::NaiveDate> {
                #release_date
            }

            #[inline]
            fn original_language(&self) -> &str {
                #original_language
            }

            #[inline]
            fn overview(&self) -> Option<&String> {
                #overview
            }

            #[inline]
            fn credit_id(&self) -> &str {
                #credit_id
            }

            #[inline]
            fn media_type(&self) -> crate::models::v3::media_type::MediaType {
                #media_type
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
