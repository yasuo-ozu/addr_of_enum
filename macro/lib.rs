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
    expr: Expr,
    #[allow(unused)]
    _comma_1: Token![,],
    variant: Ident,
    #[allow(unused)]
    _comma_2: Token![,],
    field: IdentOrNum,
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

fn to_tstr(s: &str, span: Span) -> TypeTuple {
    let mut elems = Punctuated::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                let ident = Ident::new(&format!("_{}", c), span.clone());
                elems.push(parse_quote!(::enum_offset::_tstr::#ident));
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
pub fn enum_offset(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as MacroArgs);
    quote! {
        <_ as ::enum_offset::EnumHasTagAndField<
            #{to_tstr(&args.variant.to_string(), args.variant.span())},
            #{to_tstr(&args.field.to_string(), args.field.span())}
        >>::addr_of(#{&args.expr})
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(EnumOffset)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let (g_impl, g_type, g_where) = input.generics.split_for_impl();
    let mut trait_impls = quote! {
        #[automatically_derived]
        unsafe impl #g_impl ::enum_offset::EnumOffset for #{&input.ident} #g_type #g_where {}
    };
    let mut replaced_input = input.clone();
    replaced_input.ident = Ident::new("GhostEnum", input.ident.span());
    replaced_input.variants.iter_mut().for_each(|variant| {
        match &mut variant.fields {
            Fields::Named(fields) => {
                let field_idents: Vec<_> = fields
                    .named
                    .iter()
                    .map(|field| field.ident.clone())
                    .collect();
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
                        unsafe impl #g_impl ::enum_offset::EnumHasTagAndField<
                            #{to_tstr(&variant.ident.to_string(), variant.ident.span())},
                            #{to_tstr(&field_ident.to_string(), field_ident.span())},
                        > for #{&input.ident} #g_type #g_where {
                            type Ty = #t;
                            fn addr_of(ptr: *const Self) -> *const Self::Ty {
                                // `GhostEnum` initialization will be wiped out with
                                // optimization in release mode, so it is zerocost
                                let en: GhostEnum #g_type = GhostEnum::#{&variant.ident}{
                                    #(for item in field_idents.iter()) {
                                        #item: ::core::mem::MaybeUninit::uninit(),
                                    }
                                };
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
                            unsafe impl #g_impl ::enum_offset::EnumHasTagAndField<
                                #{to_tstr(&variant.ident.to_string(), variant.ident.span())},
                                #{to_tstr(&format!("{}", nth), field.span())},
                            > for #{&input.ident} #g_type #g_where {
                                type Ty = #t;
                                fn addr_of(ptr: *const Self) -> *const Self::Ty {
                                    // `GhostEnum` initialization will be wiped out with
                                    // optimization in release mode, so it is zerocost
                                    let en: GhostEnum #g_type = GhostEnum::#{&variant.ident}(
                                        #(for _ in 0..nfields) {
                                            ::core::mem::MaybeUninit::uninit(),
                                        }
                                    );
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
