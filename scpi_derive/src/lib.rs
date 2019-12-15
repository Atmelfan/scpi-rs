extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Meta, MetaNameValue, MetaList, Lit, NestedMeta, LitByteStr};

fn get_inner_meta(list: &MetaList) -> Vec<&Meta> {
    list.nested.iter().filter_map(|nested| match *nested {
        NestedMeta::Meta(ref meta) => Some(meta),
        _ => None
    }).collect()
}

fn find_prop_bstr<'a>(meta: &'a Meta, attr: &str, property: &str) -> Option<&'a LitByteStr>{
    match meta {
        Meta::List(list) => {
            if list.path.is_ident(attr) {
                //println!("{:?}", list);
                let inner = get_inner_meta(list);

                for name_value in inner {
                    match name_value {
                        Meta::NameValue(MetaNameValue {
                                            ref path,
                                            lit: Lit::ByteStr(ref s),
                                            ..
                                        }) => {
                            if path.is_ident(property) {
                                return Some(s)
                            } else {
                                return None
                            }
                        },
                        _ => ()
                    }
                }
            }
            None
        }
        _ => None
    }
}

fn find_prop_f(meta: &Meta, attr: &str, property: &str) -> Option<f32>{
    match meta {
        Meta::List(list) => {
            if list.path.is_ident(attr) {
                //println!("{:?}", list);
                let inner = get_inner_meta(list);

                for name_value in inner {
                    match name_value {
                        Meta::NameValue(MetaNameValue {
                                            ref path,
                                            lit: Lit::Float(ref s),
                                            ..
                                        }) => {
                            if path.is_ident(property) {
                                return Some(s.base10_parse::<f32>().ok().unwrap())
                            } else {
                                return None
                            }
                        },
                        _ => ()
                    }
                }
            }
            None
        }
        _ => None
    }
}

#[proc_macro_derive(ScpiUnit, attributes(unit))]
pub fn derive_heap_size(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let variants = match input.data {
        Data::Enum(ref data) => {
            &data.variants
        }
        _ => panic!("Can only derive enum!")
    };

    let mut variant_matches = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;
        //println!(" - {} : ", variant_name.to_string());
        for attr in variant.attrs.iter() {
            let meta = attr.parse_meta().unwrap();
            if let Some(suffix) = find_prop_bstr(&meta, "unit", "suffix") {
                let multiplier = find_prop_f(&meta, "unit", "multiplier").unwrap_or(1.0f32);
                //println!("\tb\"{}\" => ({}, {}), ", String::from_utf8(suffix.value()).unwrap(), variant_name, multiplier);

                let x =  quote! {
                    #suffix => Ok((#name::#variant_name, #multiplier))
                };

                variant_matches.push(x);
            }

        }

    }

    variant_matches.push(quote!{
        _ => Err(SuffixError::Unknown)
    });


    let expanded = quote! {
        // The generated impl.
        impl  #name {
            #[doc="Returns matched suffix element unit and multiplier or an `SuffixError::UnknownSuffix` error if unsuccessful"]
            pub fn from_suffix(s: &[u8]) -> Result<(#name, f32), SuffixError> {
                match s {
                    #(#variant_matches),*
                }
            }
        }
    };

    //println!("{}", expanded);

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(ScpiError, attributes(error))]
pub fn derive_error_messages(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let variants = match input.data {
        Data::Enum(ref data) => {
            &data.variants
        }
        _ => panic!("Can only derive enum!")
    };

    let mut variant_matches = Vec::new();

    for variant in variants.iter() {
        let variant_name = &variant.ident;
        //let (_, Expr::Lit(x)) = &variant.discriminant.unwrap();

//        let code = if let Lit::Int(x) = x.lit {
//            x.to_string()
//        }else{
//            panic!("Discriminant must be an integer!")
//        };
        //println!(" - {} : ", variant_name.to_string());
        //let mut doc: Option<String> = None;
        for attr in variant.attrs.iter() {
            let meta = attr.parse_meta().unwrap();
            if let Some(message) = find_prop_bstr(&meta, "error", "message") {
                //doc = Some(format!("{:?}, \"{}\"", code, String::from_utf8(message.value()).unwrap()));
                //let multiplier = find_prop_f(&meta, "error", "multiplier").unwrap_or(1.0f32);
                //println!("\tb\"{}\" => ({}, {}), ", String::from_utf8(suffix.value()).unwrap(), variant_name, multiplier);

                let x =  quote! {
                    #name::#variant_name => Some(#message)
                };

                variant_matches.push(x);
            }

        }

    }

    variant_matches.push(quote!{
        _ => None
    });


    let expanded = quote! {
        // The generated impl.
        impl  #name {
            #[doc="Returns appropriate error message"]
            pub fn get_message(self) -> Option<&'static [u8]> {
                match self {
                    #(#variant_matches),*
                }
            }
        }
    };

    //println!("{}", expanded);

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}