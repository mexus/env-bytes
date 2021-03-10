use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::ffi::OsString;

/// For example, on Linux the macro will generate an array of `[u8]`, while on
/// Windows, it will be an array of `[u16]`.
/// ```rust
/// use env_bytes::env_bytes;
/// const NAME: [u8; 9] = env_bytes!("CARGO_PKG_NAME");
/// ```
#[proc_macro]
pub fn env_bytes(input: TokenStream) -> TokenStream {
    get_value(syn::parse_macro_input!(input as syn::Lit))
        .map(quote_value)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(target_family = "unix")]
fn quote_value(value: OsString) -> TokenStream2 {
    use std::os::unix::ffi::OsStrExt;
    let bytes = value.as_bytes();
    quote!([#(#bytes),*])
}

#[cfg(target_family = "windows")]
fn quote_value(value: CString) -> TokenStream2 {
    use std::os::windows::ffi::OsStrExt;
    let wide_bytes = value.encode_wide();
    quote!([#(#wide_bytes),*])
}

/// Reads an environment variable with the given name.
fn get_value(name: syn::Lit) -> syn::Result<OsString> {
    let (name, span) = match name {
        syn::Lit::Str(name) => {
            let span = name.span();
            (name.value(), span)
        }
        other => {
            return Err(syn::parse::Error::new(
                other.span(),
                "Only string literals are supported",
            ))
        }
    };
    std::env::var_os(&name).ok_or_else(|| {
        syn::parse::Error::new(
            span,
            format_args!(r#"Environment variable "{}" not found"#, name),
        )
    })
}
