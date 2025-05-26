use proc_macro::TokenStream;

mod is_credit;
mod make_genre;

#[proc_macro]
pub fn make_genre(input: TokenStream) -> TokenStream {
    make_genre::make_genre(input)
}

#[proc_macro_derive(IsCredit)]
pub fn is_credit(input: TokenStream) -> TokenStream {
    is_credit::is_credit(input)
}
