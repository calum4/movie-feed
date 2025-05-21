use proc_macro::TokenStream;

mod make_genre;

#[proc_macro]
pub fn make_genre(input: TokenStream) -> TokenStream {
    make_genre::make_genre(input)
}
