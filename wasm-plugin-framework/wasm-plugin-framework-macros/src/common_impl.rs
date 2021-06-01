use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, LitStr, ReturnType, Token, TraitItemMethod, parse::Parse};

pub struct CommonPluginImplementation {
    api_name: LitStr,
    api_version: LitStr,
    loader_name: Ident,
    fns: Vec<TraitItemMethod>,
}

impl Parse for CommonPluginImplementation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let api_name = input.parse()?;
        let _: Token![,] = input.parse()?;
        let api_version = input.parse()?;
        let _: Token![,] = input.parse()?;
        let loader_name = input.parse()?;
        let _: Token![,] = input.parse()?;
        let mut fns = Vec::new();
        while !input.is_empty() {
            fns.push(input.parse()?);
        }
        Ok(Self {
            api_name,
            api_version,
            loader_name,
            fns,
        })
    }
}

impl ToTokens for CommonPluginImplementation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut api_name_s = self.api_name.value();
        api_name_s.push('\0');
        let api_name_c = LitStr::new(&api_name_s, self.api_name.span());
        let api_name = &self.api_name;
        let mut api_version_s = self.api_version.value();
        api_version_s.push('\0');
        let api_version_c = LitStr::new(&api_version_s, self.api_version.span());
        let api_version = &self.api_version;
        let loader_name = &self.loader_name;

        let methods: Vec<TokenStream> = self
            .fns
            .iter()
            .cloned()
            .map(|x| {
                let sig = x.sig;
                let unsafety = sig.unsafety;
                let fn_token = sig.fn_token;
                let ident = sig.ident;
                let args = sig.inputs;
                let abi_args = args.clone().into_iter().map(|x| match x {
                    FnArg::Receiver(_) => quote!(compile_error!(
                        "Unexpected reciever (&self & co.) in arguments"
                    )),
                    FnArg::Typed(p) => {
                        let pat = p.pat;
                        quote! {(::wasm_plugin_framework::abi::into_abi(self, &#pat) as i32).into()}
                    }
                });
                let output = sig.output;
                let abi_fn_name = LitStr::new(&ident.to_string(), ident.span());

                let calling_code = quote! {
                    self.instance.exports.get_function(#abi_fn_name)
                    .expect("Tried to load a non compliant plugin, which doesn't have the one of the API's required functions")
                    .call(&[#(#abi_args),*]).expect("Unexpecteed error while calling API function")
                };

                let fn_body = match output {
                    ReturnType::Default => quote! {
                        #calling_code;
                    },
                    ReturnType::Type(_, _) => quote! {
                        ::wasm_plugin_framework::abi::from_abi(self, #calling_code[0].unwrap_i32() as u32)
                    }
                };
                quote! {
                    pub #unsafety #fn_token #ident(&self, #args) #output {
                        #fn_body
                    }
                }
            })
            .collect();

        let fns = &self.fns;

        let r = quote! {
            /// Metadata generated from the common_plugin_impl macro. Contains the api name and version.
            pub mod metadata {
                /// Null terminated version of the API name
                pub const API_NAME_C: &'static str = #api_name_c;
                pub const API_NAME: &'static str = #api_name;
                /// Null terminated version of the API version
                pub const API_VERSION_C: &'static str = #api_version_c;
                pub const API_VERSION: &'static str = #api_version;


                #[doc(hidden)]
                mod inner {
                    use super::super::*;
                    pub trait Plugin {
                        #(#fns)*
                    }
                }
                pub use inner::Plugin;
            }

            #[cfg(not(target_arch = "wasm32"))]
            mod loader {
                use ::std::path::Path;
                use ::wasm_plugin_framework::wasmer::{imports, Extern, Instance, Memory, MemoryType, Module, Store, Value, ImportObject};
                use ::wasm_plugin_framework::wasmer_wasi::WasiState;
                use super::*;

                pub struct #loader_name {
                    store: Store,
                    module: Module,
                    import_object: ImportObject,
                    pub instance: Instance,
                    pub name: String,

                }

                impl #loader_name {
                    pub fn new(bytes: &[u8]) ->Self {
                        let store = Store::default();

                        let module = Module::new(&store, bytes).expect("Error loading the WASM module based on the bytes");
                        // The module doesn't import anything, so we create an empty import object.
                        let mut wasi_env = WasiState::new("plugin")
                            // .args(&["world"])
                            // .env("KEY", "Value")
                            .finalize().expect("Error finalizing the WASI environment");

                        println!("Instantiating module with WASI imports...");
                        // Then, we get the import object related to our WASI
                        // and attach it to the Wasm instance.
                        let import_object = wasi_env.import_object(&module).expect("Error creating the WASI import object based on the wasm module");
                        let instance = Instance::new(&module, &import_object).expect("Error creating the WASM module instance");

                        let m = instance.exports.get_memory("memory").expect("Expected a memory to be exported, are you sure this is a plugin?");

                        let api_name: Vec<u8> = m
                            .view()
                            .iter()
                            .skip(
                                instance.exports.get_function("API_NAME").expect("Tried to load a non plugin, which doesn't have the API_NAME function").call(&[]).expect("Unexpected error when calling API_NAME")[0].unwrap_i32() as usize
                            )
                            .take_while(|x| {
                                let v: u8 = x.get();
                                v != 0
                            })
                            .map(|x| x.get())
                            .collect();
                        let api_name = String::from_utf8_lossy(&api_name).to_string();

                        let api_version: Vec<u8> = m
                            .view()
                            .iter()
                            .skip(
                                instance.exports.get_function("API_VERSION").expect("Tried to load a non plugin, which doesn't have the API_VERSION function").call(&[]).expect("Unexpected error when calling API_VERSION")[0].unwrap_i32() as usize
                            )
                            .take_while(|x| {
                                let v: u8 = x.get();
                                v != 0
                            })
                            .map(|x| x.get())
                            .collect();
                        let api_version = String::from_utf8_lossy(&api_version).to_string();

                        let plugin_name: Vec<u8> = m
                            .view()
                            .iter()
                            .skip(
                                instance.exports.get_function("PLUGIN_NAME").expect("Tried to load a non plugin, which doesn't have the PLUGIN_NAME function").call(&[]).expect("Unexpected error when calling PLUGIN_NAME")[0].unwrap_i32() as usize
                            )
                            .take_while(|x| {
                                let v: u8 = x.get();
                                v != 0
                            })
                            .map(|x| x.get())
                            .collect();
                        let plugin_name = String::from_utf8_lossy(&plugin_name).to_string();

                        assert_eq!(super::metadata::API_NAME, api_name, "The plugin API name doesn't match the current API name");
                        assert_eq!(super::metadata::API_VERSION, api_version, "The plugin API version doesn't match the current API version");

                        Self {
                            store,
                            module,
                            import_object,
                            instance,
                            name: plugin_name,
                        }
                    }

                    #(
                        #methods
                    )*
                }

                impl::wasm_plugin_framework::abi::PluginLoader for #loader_name {
                    fn allocate_buffer(&self, size: u32) -> u32 {
                        self.instance.exports.get_function("allocate_buffer").expect("Tried to load a non plugin, which doesn't have the allocate_buffer function").call(&[(size as i32).into()]).expect("Unexpected error when calling allocate_buffer")[0].unwrap_i32() as u32
                    }

                    fn free_buffer(&self, ptr: u32, size: u32) {
                        self.instance.exports.get_function("free_buffer").expect("Tried to load a non plugin, which doesn't have the free_buffer function").call(&[(ptr as i32).into(), (size as i32).into()]).expect("Unexpected error when calling free_buffer");
                    }

                    fn memory(&self) -> &Memory {
                        self.instance.exports.get_memory("memory").unwrap()
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub use loader::#loader_name;

        };
        r.to_tokens(tokens);
    }
}
