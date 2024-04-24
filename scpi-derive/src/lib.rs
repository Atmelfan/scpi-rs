//! This crate provides scpi derive macros
//!```ignore
//! #[derive(ScpiEnum)]
//! ```
//!
//! See [scpi - ScpiEnum](https://docs.rs/scpi/latest/scpi/option/trait.ScpiEnum.html) for details.
//!

extern crate proc_macro;

use quote::{quote, quote_spanned};
use syn::{parse_macro_input, Data, DeriveInput, LitByteStr, LitInt};

/// Derive the necessary logic to convert a enum to and from a mnemonic.
///
/// For each variant we look into the attributes looking for entry of the form #[scpi(mnemonic=b""")]
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

    // Iter over the enum variants
    for variant in variants {
        let variant_name = &variant.ident;

        // Iter over the attributes (#[repr]) of the variant
        for attr in variant.attrs.iter() {
            // We are interested only in 'scpi' attributes
            if attr.path().is_ident("scpi") {
                attr.parse_nested_meta(|meta| {

                    // For scpi attributes we look for a mnemonic name value pair as literal byte string
                    if meta.path.is_ident("mnemonic") {
                        let mnemonic: LitByteStr = meta.value()?.parse()?;

                        // We build a token stream to implement the enum creation from a mnemonic
                        let x = match &variant.fields {
                            syn::Fields::Unnamed(x) if x.unnamed.len() == 1 => quote! {
                                x if scpi::parser::mnemonic_match(#mnemonic, x) => Some(#name::#variant_name(Default::default()))
                            },
                            syn::Fields::Unit => quote! {
                                x if scpi::parser::mnemonic_match(#mnemonic, x) => Some(#name::#variant_name)
                            },
                            _ => quote_spanned! {
                                variant_name.span() => compile_error!("Variant must be unit or single unnamed field implementing default")
                            },
                        };
                        from_mnemonic_matches.push(x);

                        // We build a token stream to implement the enum conversion to a mnemonic
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
                        to_mnemonic_matches.push(x2);
                    };
                    Ok(())
                }).unwrap()
            }
        }
    }

    // Generated the impl from the collected token streams
    let expanded = quote! {
        // The generated impl.
        impl scpi::option::ScpiEnum for #name {
            fn from_mnemonic(s: &[u8]) -> Option<#name> {
                match s {
                    #(#from_mnemonic_matches),*,
                    _ => None
                }
            }

            fn mnemonic(&self) -> &'static [u8] {
                match self {
                    #(#to_mnemonic_matches),*
                }
            }
        }


        impl<'a> TryFrom<scpi::parser::tokenizer::Token<'a>> for #name {
            type Error = scpi::error::Error;

            fn try_from(value: scpi::parser::tokenizer::Token<'a>) -> scpi::error::Result<Self> {
                if let scpi::parser::tokenizer::Token::CharacterProgramData(s) = value {
                    <Self as scpi::option::ScpiEnum>::from_mnemonic(s).ok_or(scpi::error::ErrorCode::IllegalParameterValue.into())
                } else {
                    Err(scpi::error::ErrorCode::DataTypeError.into())
                }
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

/// Internal macro for scpi crate use only.
#[cfg(feature = "_private")]
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

        // Iter over all the attributes of each variant of the enum
        for attr in variant.attrs.iter() {
            // We are only interested in 'error' attributes
            if attr.path().is_ident("error") {
                attr.parse_nested_meta(|meta| {
                    // We look for three distinct type of name value pairs

                    // 'message' with a byte string value
                    if meta.path.is_ident("message") {
                        let message: LitByteStr = meta.value()?.parse()?;

                        let x = quote! {
                            #name::#variant_name => #message
                        };
                        variant_matches.push(x);
                        return Ok(());
                    }

                    // 'code' with an integer value
                    if meta.path.is_ident("code") {
                        let code: LitInt = meta.value()?.parse()?;

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
                        return Ok(());
                    }

                    // 'custom' with arbitrary tokens
                    if meta.path.is_ident("custom") {
                        let x = quote! {
                            #name::#variant_name(_,msg) => msg
                        };
                        variant_matches.push(x);
                        let cx = quote! {
                            #name::#variant_name(code,_) => code
                        };
                        //println!("--- {}", cx);
                        code_variant_matches.push(cx);
                        return Ok(());
                    }

                    Ok(())
                })
                .expect(&format!("{:?}", variant_name.to_string()));
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
