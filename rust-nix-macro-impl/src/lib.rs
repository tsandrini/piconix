use proc_macro::TokenStream;
use syn::spanned::Spanned;

#[proc_macro]
pub fn nix(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();
    let span = tokens.span();
    let code_as_string = tokens.to_string();

    let ast = match rust_nix_core::parser::parse(&code_as_string) {
        Ok(ast) => ast,
        Err(e) => {
            // Use the span we captured earlier for a precise error location.
            return syn::Error::new(span, format!("Nix parsing failed:\n{}", e))
                .to_compile_error()
                .into();
        }
    };

    rust_nix_core::codegen::generate_token_stream(&ast).into()
}
