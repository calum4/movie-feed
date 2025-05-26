use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token, parse_macro_input};
use unicode_segmentation::UnicodeSegmentation;

struct MakeGenreInput {
    enum_ident: Ident,
    json: LitStr,
}

impl Parse for MakeGenreInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let enum_ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let json: LitStr = input.parse()?;

        Ok(Self { enum_ident, json })
    }
}

#[derive(Deserialize)]
struct GenresJson {
    genres: Vec<GenresElement>,
}

#[derive(Deserialize)]
struct GenresElement {
    id: usize,
    name: String,
}

fn sanitise_ident(name: String) -> String {
    name.split(|c: char| c.is_whitespace() || matches!(c, '_' | '-'))
        .filter(|str| !str.is_empty())
        .map(|name| {
            let mut graphemes = name.graphemes(true);

            [
                graphemes
                    .next()
                    .expect("empty strings are filtered from the iterator")
                    .to_uppercase(),
                graphemes.as_str().to_lowercase(),
            ]
            .concat()
        })
        .collect::<String>()
        .replace('&', "And")
}

pub(crate) fn make_genre(input: TokenStream) -> TokenStream {
    let MakeGenreInput { enum_ident, json } = parse_macro_input!(input as MakeGenreInput);

    let json: GenresJson = serde_json::from_str(json.value().as_str())
        .expect(r#"incorrect json format, must be `{"genres": [{"id": 0, "name": "example"}]}`"#);

    let (genre_ids, (genre_names, genre_idents)): (Vec<_>, (Vec<_>, Vec<_>)) = json
        .genres
        .into_iter()
        .map(|elem| {
            (
                elem.id,
                (
                    elem.name.clone(),
                    format_ident!("{}", sanitise_ident(elem.name)),
                ),
            )
        })
        .unzip();

    let expanded = quote! {
        #[non_exhaustive]
        #[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum #enum_ident {
            #(#genre_idents,)*
            Unknown(usize),
        }

        impl crate::models::v3::genres::Genre for #enum_ident {
            fn id(&self) -> crate::models::v3::genre_id::GenreId {
                match *self {
                    #(#enum_ident::#genre_idents => #genre_ids,)*
                    #enum_ident::Unknown(id) => id,
                }.into()
            }

            fn name(&self) -> &'static str {
                match self {
                    #(Self::#genre_idents => #genre_names,)*
                    Self::Unknown(_) => "Unknown Genre",
                }
            }
        }

        impl From<crate::models::v3::genre_id::GenreId> for #enum_ident {
            fn from(genre_id: crate::models::v3::genre_id::GenreId) -> Self {
                match *genre_id {
                    #(#genre_ids => Self::#genre_idents,)*

                    id => Self::Unknown(id),
                }
            }
        }

        impl From<#enum_ident> for crate::models::v3::genre_id::GenreId {
            fn from(genre: #enum_ident) -> Self {
                genre.id()
            }
        }

        impl Display for #enum_ident {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.name())
            }
        }
    };

    TokenStream::from(expanded)
}
