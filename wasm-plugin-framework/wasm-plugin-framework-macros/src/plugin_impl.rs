use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, ItemFn, LitStr, Path, ReturnType, Token, parse::Parse};

pub struct PluginImplementation {
	common_lib: Path,
	plugin_name: LitStr,
	fns: Vec<ItemFn>
}

impl Parse for PluginImplementation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let common_lib = input.parse()?;
		let _: Token![,] = input.parse()?;
        let plugin_name = input.parse()?;
		let _: Token![,] = input.parse()?;
		let mut fns = Vec::new();
		while !input.is_empty() {
			fns.push(input.parse()?)
		}

		Ok(Self {
			common_lib, plugin_name, fns
		})
    }
}

impl ToTokens for PluginImplementation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
		let common_lib = &self.common_lib;
		let mut plugin_name_s = self.plugin_name.value();
		plugin_name_s.push('\0');
		let plugin_name = LitStr::new(&plugin_name_s, self.plugin_name.span());
		let fns = &self.fns;
		let abi_fns: Vec<_> = self.fns.iter().map(|a| {
			let fn_token = &a.sig.fn_token;
			let ident = &a.sig.ident;
			let abi_args_compound: Vec<_> = a.sig.inputs.clone().into_iter().enumerate().map(|(i, x)| match x {
				FnArg::Receiver(_) => unreachable!(),
				FnArg::Typed(_) => {
					let argname = quote::format_ident!("arg{}", i);
					(quote!(#argname: u32), quote! {#common_lib::wasm_plugin_framework::abi::from_abi(#argname)})
				}
			}).collect();
			let mut abi_args = Vec::with_capacity(abi_args_compound.len());
			let mut abi_args_conversions = Vec::with_capacity(abi_args_compound.len());

			for (a, b) in abi_args_compound {
				abi_args.push(a);
				abi_args_conversions.push(b);
			}

			let calling_code = quote! {
				Plugin::#ident(#(#abi_args_conversions),*)
			};

			let (body, return_t) = match &a.sig.output {
				ReturnType::Default => (quote! {
					#calling_code;
				}, quote!()),
				ReturnType::Type(_, _) => (quote! {
					#common_lib::wasm_plugin_framework::abi::into_abi(&#calling_code)
				}, quote!(-> u32))
			};

			quote!{
				#[no_mangle]
				pub extern "C" #fn_token #ident(#(#abi_args),*) #return_t {#body}
			}
		}).collect();
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

			mod plugin_impl {
				use super::*;

				use #common_lib::metadata::Plugin as PluginTrait;
				struct Plugin;

				impl PluginTrait for Plugin {
					#(#fns)*
				}

				#(#abi_fns)*
			}
			use plugin_impl::*;
		};
		r.to_tokens(tokens);
    }
}