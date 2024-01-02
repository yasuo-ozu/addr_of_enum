//! Use [`addr_of_enum crate`](//crates.io/crates/addr_of_enum) instead.

use derive_syn_parse::Parse;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::*;
use template_quote::quote;

#[derive(Parse)]
struct MacroArgs {
    krate: Path,
    #[allow(unused)]
    _comma_0: Token![,],
    name: IdentOrNum,
}

#[derive(Parse, Debug)]
enum IdentOrNum {
    #[peek(Ident, name = "Ident")]
    Ident(Ident),
    #[peek(LitInt, name = "Num")]
    Num(LitInt),
}

impl Spanned for IdentOrNum {
    fn span(&self) -> Span {
        match self {
            Self::Ident(ident) => ident.span(),
            Self::Num(litint) => litint.span(),
        }
    }
}

impl core::fmt::Display for IdentOrNum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ident(ident) => ident.fmt(f),
            Self::Num(litint) => litint.fmt(f),
        }
    }
}

fn to_tstr(krate: &Path, s: &str, span: Span) -> TypeTuple {
    let mut elems = Punctuated::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                let ident = Ident::new(&format!("_{}", c), span.clone());
                elems.push(parse_quote!(#krate::_tstr::#ident));
            }
            _ => abort!(span, "Bad char '{}'", c),
        }
    }
    TypeTuple {
        paren_token: Default::default(),
        elems,
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn get_tstr(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as MacroArgs);
    quote! {
        #{to_tstr(&args.krate, &args.name.to_string(), args.name.span())}
    }
    .into()
}

/// This macro impls [`AddrOfEnum`] trait. It works only on enums.
#[proc_macro_error]
#[proc_macro_derive(AddrOfEnum, attributes(addr_of_enum))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let krate: Path = input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("addr_of_enum"))
        .map(|attr| {
            attr.parse_args::<Path>()
                .unwrap_or_else(|e| abort!(attr.span(), "Bad input: {}", e))
        })
        .unwrap_or(parse_quote!(::addr_of_enum));
    let (g_impl, g_type, g_where) = input.generics.split_for_impl();
    let mut trait_impls = quote! {
        #[automatically_derived]
        unsafe impl #g_impl #krate::AddrOfEnum for #{&input.ident} #g_type #g_where {}
    };
    let mut replaced_input = input.clone();
    replaced_input.attrs = replaced_input
        .attrs
        .into_iter()
        .filter(|attr| !attr.path.is_ident("addr_of_enum"))
        .collect();
    replaced_input.ident = Ident::new("GhostEnum", input.ident.span());
    replaced_input.variants.iter_mut().for_each(|variant| {
        // `GhostEnum` initialization will be wiped out with
        // optimization in release mode, so it is zerocost
        let initializer = match &variant.fields {
            Fields::Named(_) => {
                quote! {
                    GhostEnum::#{&variant.ident} {
                        #(for field in variant.fields.iter()) {
                            #{&field.ident}: ::core::mem::MaybeUninit::uninit(),
                        }
                    }
                }
            }
            Fields::Unnamed(_) => {
                quote! {
                    GhostEnum::#{&variant.ident} (
                        #(for _ in variant.fields.iter()) {::core::mem::MaybeUninit::uninit(),}
                    )
                }
            }
            _ => quote! {GhostEnum},
        };
        trait_impls = quote!{
            #trait_impls
            unsafe impl #g_impl #krate::EnumHasTag<
                #{to_tstr(&krate, &variant.ident.to_string(), variant.ident.span())},
            > for #{&input.ident} #g_type #g_where {
                fn discriminant() -> core::mem::Discriminant<Self> {
                    let val: GhostEnum #g_type = #initializer;
                    /// SAFETY: both has same memory layout
                    unsafe {
                        ::core::mem::transmute(::core::mem::discriminant(&val))
                    }
                }
            }
        };
        match &mut variant.fields {
            Fields::Named(fields) => {
                fields.named.iter_mut().for_each(|field| {
                    let t = field.ty.clone();
                    // Replace type `T` with `MaybeUninit<T>`, which has the same memory
                    // layout, but not need to be initialized.
                    // If `T` is uninhabited type, also `MaybeUninit<T>` is
                    // uninhabited. It works well to keep the memory layout of
                    // `GhostEnum` same as the original enum.
                    let replaced_ty = parse_quote! {::core::mem::MaybeUninit<#t>};
                    let field_ident = field.ident.as_ref().unwrap();
                    trait_impls = quote! {
                        #trait_impls
                        #[automatically_derived]
                        unsafe impl #g_impl #krate::EnumHasTagAndField<
                            #{to_tstr(&krate, &variant.ident.to_string(), variant.ident.span())},
                            #{to_tstr(&krate, &field_ident.to_string(), field_ident.span())},
                        > for #{&input.ident} #g_type #g_where {
                            type Ty = #t;
                            fn addr_of(ptr: *const Self) -> *const Self::Ty {
                                let en: GhostEnum #g_type = #initializer;
                                match &en {
                                    GhostEnum::#{&variant.ident} {
                                        #{&field.ident},
                                        ..
                                    } => unsafe {
                                        ptr.cast::<u8>().offset(
                                            (#{&field.ident} as *const #replaced_ty as isize)
                                            - (&en as *const GhostEnum #g_type as isize)
                                        ).cast()
                                    }
                                    _ => unsafe {
                                        ::core::hint::unreachable_unchecked()
                                    }
                                }
                            }
                        }
                    };
                    field.ty = replaced_ty;
                })
            }
            Fields::Unnamed(fields) => {
                let nfields = fields.unnamed.len();
                fields
                    .unnamed
                    .iter_mut()
                    .enumerate()
                    .for_each(|(nth, field)| {
                        let t = field.ty.clone();
                        let replaced_ty = parse_quote! {::core::mem::MaybeUninit<#t>};
                        trait_impls = quote! {
                            #trait_impls
                            #[automatically_derived]
                            unsafe impl #g_impl #krate::EnumHasTagAndField<
                                #{to_tstr(&krate, &variant.ident.to_string(), variant.ident.span())},
                                #{to_tstr(&krate, &format!("{}", nth), field.span())},
                            > for #{&input.ident} #g_type #g_where {
                                type Ty = #t;
                                fn addr_of(ptr: *const Self) -> *const Self::Ty {
                                    let en: GhostEnum #g_type = #initializer;
                                    match &en {
                                        GhostEnum::#{&variant.ident} (
                                            #(for _ in 0..nth) { _, }
                                            var,
                                            #(for _ in (nth+1)..nfields) { _, }
                                        ) => unsafe {
                                            ptr.cast::<u8>().offset(
                                                (var as *const #replaced_ty as isize)
                                                - (&en as *const GhostEnum #g_type as isize)
                                            ).cast()
                                        }
                                        _ => unsafe {
                                            ::core::hint::unreachable_unchecked()
                                        }
                                    }
                                }
                            }

                        };
                        field.ty = replaced_ty;
                    });
            }
            _ => (),
        }
    });
    quote! { const _: () = { #trait_impls #replaced_input }; }.into()
}
