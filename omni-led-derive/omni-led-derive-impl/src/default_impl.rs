use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields};

use crate::common::{get_attribute_with_default_value, parse_attributes};

pub fn expand_default_impl_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let default_initializers = generate_initializers(input.data);

    let expanded = quote! {
        impl Default for #name {
            fn default() -> Self {
                Self {
                    #default_initializers
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn generate_initializers(data: Data) -> TokenStream {
    let Data::Struct(struct_data) = data else {
        panic!("Expected a struct");
    };
    let Fields::Named(fields) = struct_data.fields else {
        panic!("Expected named fields");
    };

    let default_initializers = fields.named.into_iter().map(|f| {
        let ident = f.ident.unwrap();
        let attrs = get_field_attributes(&f.attrs);
        let default = attrs.default;

        quote! { #ident: #default, }
    });

    quote! { #(#default_initializers)* }
}

struct FieldAttributes {
    default: TokenStream,
}

fn get_field_attributes(attributes: &Vec<Attribute>) -> FieldAttributes {
    let mut attributes = parse_attributes("omni", attributes);

    FieldAttributes {
        default: get_attribute_with_default_value(
            &mut attributes,
            "default",
            quote!(Default::default()),
        )
        .expect("\"default\" attribute is required"),
    }
}
