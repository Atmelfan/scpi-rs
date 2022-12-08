extern crate proc_macro;

use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, Data, DeriveInput, Lit, LitByteStr, LitInt, Meta, MetaList, MetaNameValue,
    NestedMeta,
};

fn get_inner_meta(list: &MetaList) -> Vec<&Meta> {
    list.nested
        .iter()
        .filter_map(|nested| match *nested {
            NestedMeta::Meta(ref meta) => Some(meta),
            _ => None,
        })
        .collect()
}

fn find_prop_bstr<'a>(meta: &'a Meta, attr: &str, property: &str) -> Option<&'a LitByteStr> {
    if let Meta::List(list) = meta {
        if list.path.is_ident(attr) {
            //println!("{:?}", list);
            let inner = get_inner_meta(list);

            for name_value in inner {
                if let Meta::NameValue(MetaNameValue {
                    ref path,
                    lit: Lit::ByteStr(ref s),
                    ..
                }) = name_value
                {
                    if path.is_ident(property) {
                        return Some(s);
                    }
                }
            }
        }
    }
    None
}

fn find_prop_bint<'a>(meta: &'a Meta, attr: &str, property: &str) -> Option<&'a LitInt> {
    if let Meta::List(list) = meta {
        if list.path.is_ident(attr) {
            //println!("{:?}", list);
            let inner = get_inner_meta(list);

            for name_value in inner {
                if let Meta::NameValue(MetaNameValue {
                    ref path,
                    lit: Lit::Int(ref s),
                    ..
                }) = name_value
                {
                    if path.is_ident(property) {
                        return Some(s);
                    }
                }
            }
        }
    }
    None
}

fn find_prop_path<'a>(meta: &'a Meta, attr: &str, property: &str) -> bool {
    if let Meta::List(list) = meta {
        if list.path.is_ident(attr) {
            //println!("{:?}", list);
            let inner = get_inner_meta(list);

            for name_value in inner {
                if let Meta::Path(path) = name_value {
                    return path.is_ident(property);
                }
            }
        }
    }
    false
}

/// Derive the necessary logic to convert a enum to and from a mnemonic.
#[proc_macro_derive(ScpiEnum, attributes(scpi))]
pub fn derive_scpi_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let variants = match input.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("Can only derive enum!"),
    };

    let mut from_mnemonic_matches = Vec::new();
    let mut to_mnemonic_matches = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;
        //println!(" - {} : ", variant_name.to_string());
        for attr in variant.attrs.iter() {
            let meta = attr.parse_meta().unwrap();
            if let Some(mnemonic) = find_prop_bstr(&meta, "scpi", "mnemonic") {
                // Check that mnemonic is not empty and start with a uppercase character
                let shortform_len = mnemonic
                    .value()
                    .iter()
                    .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                    .count();
                let (last_upper, _) = mnemonic
                    .value()
                    .iter()
                    .enumerate()
                    .rfind(|(_, c)| c.is_ascii_uppercase() || c.is_ascii_digit())
                    .expect("Mnemonic cannot be empty");

                if shortform_len < 1
                    || last_upper != shortform_len - 1
                    || mnemonic.value().len() > 12
                {
                    panic!("{}::{} Invalid mnemonic, must follow \"SHORTlong\" format and <= 12 characters", name, variant_name);
                }
                let x = match &variant.fields {
                    syn::Fields::Unnamed(x) if x.unnamed.len() == 1 => quote! {
                        x if scpi::util::mnemonic_compare(#mnemonic, x) => Some(#name::#variant_name(Default::default()))
                    },
                    syn::Fields::Unit => quote! {
                        x if scpi::util::mnemonic_compare(#mnemonic, x) => Some(#name::#variant_name)
                    },
                    _ => quote_spanned! {
                        variant_name.span() => compile_error!("Variant must be unit or single unnamed field implementing default")
                    },
                };
                from_mnemonic_matches.push(x);

                let mnemonic_return = LitByteStr::new(&mnemonic.value(), variant_name.span());

                let x2 = match &variant.fields {
                    syn::Fields::Unnamed(x) if x.unnamed.len() == 1 => quote! {
                        #name::#variant_name(..) => #mnemonic_return
                    },
                    syn::Fields::Unit => quote! {
                        #name::#variant_name => #mnemonic_return
                    },
                    _ => quote_spanned! {
                        variant_name.span() => compile_error!("Variant must be unit or single unnamed field implementing default")
                    },
                };
                //println!("{}", x2);
                to_mnemonic_matches.push(x2);
            } else {
                panic!("{}::{} Missing mnemonic", name, variant_name);
            }
        }
    }
    from_mnemonic_matches.push(quote! {
        _ => None
    });

    let expanded = quote! {
        // The generated impl.
        impl scpi::option::ScpiEnum for #name {
            fn from_mnemonic(s: &[u8]) -> Option<#name> {
                match s {
                    #(#from_mnemonic_matches),*
                }
            }

            fn mnemonic(&self) -> &'static [u8] {
                match self {
                    #(#to_mnemonic_matches),*
                }
            }
        }


        impl<'a> TryFrom<scpi::tokenizer::Token<'a>> for #name {
            type Error = scpi::error::Error;

            fn try_from(value: scpi::tokenizer::Token<'a>) -> scpi::error::Result<Self> {
                if let scpi::tokenizer::Token::CharacterProgramData(s) = value {
                    Self::from_mnemonic(s).ok_or(scpi::error::ErrorCode::IllegalParameterValue.into())
                } else {
                    Err(scpi::error::ErrorCode::DataTypeError.into())
                }
            }
        }
    };

    //println!("{}", expanded);

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

#[cfg(feature = "__private")]
#[proc_macro_derive(ScpiError, attributes(error))]
pub fn derive_error_messages(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let variants = match input.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("Can only derive enum!"),
    };

    let mut variant_matches = Vec::new();
    let mut code_variant_matches = Vec::new();
    let mut variant_code_matches = Vec::new();

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

                let x = quote! {
                    #name::#variant_name => #message
                };
                variant_matches.push(x);
            }
            if let Some(code) = find_prop_bint(&meta, "error", "code") {
                //doc = Some(format!("{:?}, \"{}\"", code, String::from_utf8(message.value()).unwrap()));
                //let multiplier = find_prop_f(&meta, "error", "multiplier").unwrap_or(1.0f32);
                let cx = quote! {
                    #name::#variant_name => #code
                };
                //println!("--- {}", cx);
                //compile_error!("bint");
                code_variant_matches.push(cx);

                let ccx = quote! {
                    #code => Some(#name::#variant_name)
                };
                variant_code_matches.push(ccx);
            }
            if find_prop_path(&meta, "error", "custom") {
                let x = quote! {
                    #name::#variant_name(_,msg) => msg
                };
                variant_matches.push(x);
                let cx = quote! {
                    #name::#variant_name(code,_) => code
                };
                //println!("--- {}", cx);
                code_variant_matches.push(cx);
            }
        }
    }

    let expanded = quote! {
        // The generated impl.
        impl  #name {
            #[doc="Returns appropriate error message"]
            pub fn get_message(self) -> &'static [u8] {
                match self {
                    #(#variant_matches),*
                }
            }

            #[doc="Returns appropriate error code"]
            pub fn get_code(self) -> i16 {
                match self {
                    #(#code_variant_matches),*
                }
            }

            #[doc="Returns appropriate error from code (if any)"]
            pub fn get_error(code: i16) -> Option<Self> {
                match code {
                    #(#variant_code_matches),*,
                    _ => None
                }
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
