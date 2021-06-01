use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Path, LitStr, Token, parse::Parse};

pub struct PluginImplementation {
	common_lib: Path,
	plugin_name: LitStr
}

impl Parse for PluginImplementation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let common_lib = input.parse()?;
		let _: Token![,] = input.parse()?;
        let plugin_name = input.parse()?;

		Ok(Self {
			common_lib, plugin_name
		})
    }
}

impl ToTokens for PluginImplementation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
		let common_lib = &self.common_lib;
		let mut plugin_name_s = self.plugin_name.value();
		plugin_name_s.push('\0');
		let plugin_name = LitStr::new(&plugin_name_s, self.plugin_name.span());
        let r = quote!{
			/// The plugin metadata functions (eg. name)
			pub mod metadata {
				#[no_mangle]
				pub extern "C" fn API_NAME() -> *const u8 {
					#common_lib::metadata::API_NAME_C.as_ptr()
				}

				#[no_mangle]
				pub extern "C" fn API_VERSION() -> *const u8 {
					#common_lib::metadata::API_VERSION_C.as_ptr()
				}

				#[no_mangle]
				pub extern "C" fn PLUGIN_NAME() -> *const u8 {
					#plugin_name.as_ptr()
				}
			}
			// So that the names dont collide, as the modules dont matter when exporting
			use metadata::*;
		};
		r.to_tokens(tokens);
    }
}