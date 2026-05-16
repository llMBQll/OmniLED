use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemFn};

pub fn expand_plugin_entry_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut inner: ItemFn = syn::parse2(item).unwrap();

    let fn_name = inner.sig.ident.clone();
    let is_async = inner.sig.asyncness.is_some();

    let inner_name = Ident::new(&format!("__rust_{}", fn_name), Span::call_site());
    inner.sig.ident = inner_name.clone();

    let call_inner = if is_async {
        quote! {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    #inner_name(api, args).await
                })
        }
    } else {
        quote! { #inner_name(api, args) }
    };

    let expanded = quote! {
        #inner

        #[unsafe(no_mangle)]
        pub extern "C" fn #fn_name(
            c_api: omni_led_api::c_api::OmniLedApi,
            argc:  ::std::os::raw::c_int,
            argv:  *mut *mut ::std::os::raw::c_char,
        ) {
            let api = omni_led_api::rust_api::OmniLedApi::new(c_api);

            let args = unsafe { ::std::slice::from_raw_parts(argv, argc as usize) };
            let args = args
                .iter()
                .map(|arg| unsafe {
                    ::std::ffi::CStr::from_ptr(*arg)
                        .to_str()
                        .expect("Invalid UTF-8")
                })
                .collect();

            #call_inner
        }
    };

    expanded.into()
}
