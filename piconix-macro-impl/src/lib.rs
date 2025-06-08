use proc_macro::TokenStream;
use syn::spanned::Spanned;

#[proc_macro]
pub fn nix(input: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = input.into();
    let span = tokens.span(); // Used for error reporting

    let root = std::env::current_dir().expect("Could not get current working directory");

    let code_as_string = tokens.to_string();

    let ast = match piconix_core::parser::parse(&code_as_string, &root) {
        Ok(ast) => ast,
        Err(e) => {
            // Use the span we captured earlier for a precise error location.
            return syn::Error::new(span, format!("Nix parsing failed:\n{}", e))
                .to_compile_error()
                .into();
        }
    };

    piconix_core::codegen::generate_token_stream(&ast).into()
}
