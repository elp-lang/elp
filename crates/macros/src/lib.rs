#[proc_macro_derive(FromCST)]
pub fn from_cst_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = &ast.ident;

    let gen = quote! {
        impl FromCST for #name {
            fn from_cst(cst: &CSTExpression) -> Self {
                // Implement conversion logic here
            }
        }
    };

    gen.into()
}
