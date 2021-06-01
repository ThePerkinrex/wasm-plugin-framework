use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;



mod common_impl;


#[proc_macro]
/// The common plugin implementation, to be placed in the common library, used by both host and plugin
/// It consists for now of a name, a version and the plugin struct identifier.
/// ```
/// common_plugin_implementation!("API NAME", "0.1.0", Plugin)
/// ```
pub fn common_plugin_implementation(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as common_impl::CommonPluginImplementation);
    let r = quote!{
        #input
    };
    TokenStream::from(r)
}


mod plugin_impl;


#[proc_macro]
/// The plugin implementation.
/// It takes the common library in which the common_plugin_implementation was used, the plugin name, and the consts and functions required by the plugin
pub fn plugin(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as plugin_impl::PluginImplementation);
    let r = quote!{
        #input
    };
    TokenStream::from(r)
}


mod common_type_impl;


// #[proc_macro_attribute]
// /// A way to pass custom type through the FFI boundary
// pub fn common_type(tokens: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(tokens as plugin_impl::PluginImplementation);
//     let r = quote!{
//         #input
//     };
//     TokenStream::from(r)
// }