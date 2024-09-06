use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn exocore_app(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);

    #[allow(clippy::redundant_clone)]
    let struct_ident = input_struct.ident.clone();

    TokenStream::from(quote! {
        #input_struct

        #[no_mangle]
        pub extern "C" fn __exocore_app_init() {
            let instance = <#struct_ident>::new();
            ::exocore::apps::sdk::app::__exocore_app_register(Box::new(instance));
        }
    })
}
