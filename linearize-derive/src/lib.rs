use {
    proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree},
    quote::{quote, quote_spanned},
    syn::{
        parse::{Parse, ParseStream},
        parse_macro_input, parse_quote,
        spanned::Spanned,
        Attribute, Error, Generics, Item, ItemEnum, ItemStruct, LitInt, Path, Token, Type,
    },
};

/// A proc macro to derive the `Linearize` trait.
///
/// This macro can be used to derive the `Linearize` trait for structs and enums.
///
/// The structure of these types can be arbitrary except that all contained fields must
/// also implement the `Linearize` trait.
///
/// # Using different crate names
///
/// If you use the linearize crate under a name other than `linearize`, you can use the
/// `crate` attribute to have the proc macro reference the correct crate. For example,
/// if you import the linearize crate like this:
///
/// ```toml
/// linearize-0_1 = { package = "linearize", version = "0.1" }
/// ```
///
/// Then you can use this attribute as follows:
///
/// ```rust,ignore
/// #[derive(Linearize)]
/// #[linearize(crate = linearize_0_1)]
/// struct S;
/// ```
///
/// <div class="warning">
///
/// If you import the linearize crate under a name other than `linearize` or use the crate
/// attribute, you must ensure that these two names are in sync. Otherwise the macro
/// might not uphold the invariants of the `Linearize` trait.
///
/// </div>
///
/// # Implementing const functions
///
/// If you want to use the forms of the `static_map` and `static_copy_map` macros that
/// work in constants and statics, you must enable the `const` attribute:
///
/// ```rust,ignore
/// #[derive(Linearize)]
/// #[linearize(const)]
/// struct S;
/// ```
///
/// In this case, your type must only contain fields that also enabled this attribute. In
/// particular, you cannot use any of the standard types `u8`, `bool`, etc.
///
/// # Performance
///
/// If the type is a C-style enum with default discriminants, the derived functions will
/// be compiled to a jump table in debug mode and will be completely optimized away in
/// release mode.
///
/// If the type contains fields, the generated code will still be reasonably efficient.
///
/// # Limitations
///
/// While this macro fully supports types with generics, the generated output will not
/// compile. This is due to limitations of the rust type system. If a future version of
/// the rust compiler lifts these limitations, this macro will automatically start working
/// for generic types.
#[proc_macro_derive(Linearize, attributes(linearize))]
pub fn derive_linearize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input: Input = parse_macro_input!(input as Input);
    let crate_name = &input.attributes.crate_name;
    let FullyLinearized {
        linearize,
        delinearize,
        const_linearize,
        const_delinearize,
        const_names,
        consts,
        max_len,
    } = input.build_linearize();
    let where_clause = input.generics.make_where_clause();
    for ty in &input.critical_types {
        where_clause
            .predicates
            .push(parse_quote!(#ty: #crate_name::Linearize));
    }
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let ident = input.ident;
    let mut const_impl = quote! {};
    if input.attributes.enable_const {
        const_impl = quote! {
            #[doc(hidden)]
            impl #impl_generics #ident #type_generics #where_clause {
                #[inline]
                pub const fn __linearize_d66aa8fa_6974_4651_b2b7_75291a9e7105(&self) -> usize {
                    #const_linearize
                }

                #[inline]
                pub const unsafe fn __from_linear_unchecked_fb2f0b31_5b5a_48b4_9264_39d0bdf94f1d(linear: usize) -> Self {
                    #const_delinearize
                }
            }
        };
    }
    let res = quote_spanned! { input.span =>
        #[allow(clippy::modulo_one, clippy::manual_range_contains)]
        const _: () = {
            trait __C {
                #(const #const_names: usize;)*
            }

            impl #impl_generics __C for #ident #type_generics #where_clause {
                #(#consts)*
            }

            // SAFETY:
            //
            // Storage and CopyStorage obviously are the required type.
            //
            // The bodies if `linearize` and `from_linear_unchecked` are generated as follows:
            //
            // First, consider a struct s = { a1: T1, ..., an: Tn }. The calculated LENGTH
            // is the product of the lengths of the Ti. We write |T| for the LENGTH of T.
            // Write Bi = |T{i+1}| * ... * |Tn|, the product of the LENGTHs of the later types.
            // Write linear(v) for the linearization of v. Then we define
            // linear(s) = \sum_{i} linear(ai) * Bi.
            // It is easy to see that linear(s) / Bi % Ti = linear(ai).
            // Therefore we have created a bijection between the struct and [0, B0).
            //
            // Now consider an enum e = { V1, ..., Vn } where each variant can have fields.
            // Each Vi can be treated like a struct and we can define a bijection between
            // the enum and [0, |V1| + ... + |Vn|) by mapping V1 to [0, |V1|), V2 to
            // [|V1|, |V1| + |V2|), and so on.
            #[automatically_derived]
            unsafe impl #impl_generics
            #crate_name::Linearize for #ident #type_generics
            #where_clause
            {
                type Storage<__T> = [__T; <Self as #crate_name::Linearize>::LENGTH];

                type CopyStorage<__T> = [__T; <Self as #crate_name::Linearize>::LENGTH] where __T: Copy;

                const LENGTH: usize = <Self as __C>::#max_len;

                #[inline]
                fn linearize(&self) -> usize {
                    #linearize
                }

                #[inline]
                unsafe fn from_linear_unchecked(linear: usize) -> Self {
                    #delinearize
                }
            }

            #const_impl
        };
    };
    res.into()
}

struct Input {
    span: Span,
    ident: Ident,
    generics: Generics,
    critical_types: Vec<Type>,
    kind: Kind,
    attributes: InputAttributes,
}

struct InputAttributes {
    crate_name: Path,
    enable_const: bool,
}

#[derive(Default)]
struct InputAttributesOpt {
    crate_name: Option<Path>,
    enable_const: bool,
}

enum Kind {
    Struct(StructInput),
    Enum(EnumInput),
}

struct StructInput {
    fields: Vec<StructField>,
}

struct EnumInput {
    variants: Vec<EnumVariant>,
}

struct EnumVariant {
    ident: Ident,
    fields: Vec<StructField>,
}

struct PartialLinearized {
    linearize: TokenStream,
    delinearize: TokenStream,
    const_linearize: TokenStream,
    const_delinearize: TokenStream,
    max_len: Option<TokenStream>,
}

struct FullyLinearized {
    linearize: TokenStream,
    delinearize: TokenStream,
    const_linearize: TokenStream,
    const_delinearize: TokenStream,
    const_names: Vec<Ident>,
    consts: Vec<TokenStream>,
    max_len: Ident,
}

struct StructField {
    original_name: Option<Ident>,
    generated_name: Option<Ident>,
    ty: Type,
}

fn build_linearize_struct(
    input: &Input,
    fields: &[StructField],
    base: &Ident,
) -> PartialLinearized {
    let crate_name = &input.attributes.crate_name;
    let mut linearize_parts = vec![];
    let mut delinearize_parts = vec![];
    let mut const_linearize_parts = vec![];
    let mut const_delinearize_parts = vec![];
    let mut max_len = quote!(1usize);
    for (idx, field) in fields.iter().enumerate().rev() {
        let idx = LitInt::new(&idx.to_string(), Span::call_site());
        let ref_name = match &field.generated_name {
            Some(i) => quote! {#i},
            None => match &field.original_name {
                Some(i) => quote! { &self.#i },
                None => quote! { &self.#idx },
            },
        };
        let mut_name = match &field.original_name {
            Some(i) => quote! { #i },
            None => quote! { #idx },
        };
        let ty = &field.ty;
        linearize_parts.push(quote! {
            res = res.wrapping_add(<#ty as #crate_name::Linearize>::linearize(#ref_name).wrapping_mul(const { #max_len }));
        });
        delinearize_parts.push(quote! {
            #mut_name: {
                let idx = (linear / const { #max_len }) % <#ty as #crate_name::Linearize>::LENGTH;
                <#ty as #crate_name::Linearize>::from_linear_unchecked(idx)
            },
        });
        if input.attributes.enable_const {
            const_linearize_parts.push(quote! {
                res = res.wrapping_add(<#ty>::__linearize_d66aa8fa_6974_4651_b2b7_75291a9e7105(#ref_name).wrapping_mul(const { #max_len }));
            });
            const_delinearize_parts.push(quote! {
                #mut_name: {
                    let idx = (linear / const { #max_len }) % <#ty as #crate_name::Linearize>::LENGTH;
                    <#ty>::__from_linear_unchecked_fb2f0b31_5b5a_48b4_9264_39d0bdf94f1d(idx)
                },
            });
        }
        max_len = quote! {
            #max_len * <#ty as #crate_name::Linearize>::LENGTH
        };
    }
    delinearize_parts.reverse();
    const_delinearize_parts.reverse();
    let make_linearize = |parts: &[TokenStream]| {
        if fields.is_empty() {
            quote! { <Self as __C>::#base }
        } else {
            quote! {
                let mut res = <Self as __C>::#base;
                #(#parts)*
                res
            }
        }
    };
    let make_delinearize = |parts: &[TokenStream]| {
        quote! {
            { #(#parts)* }
        }
    };
    let mut max_len = Some(max_len);
    if fields.is_empty() {
        max_len = None;
    }
    PartialLinearized {
        linearize: make_linearize(&linearize_parts),
        delinearize: make_delinearize(&delinearize_parts),
        const_linearize: make_linearize(&const_linearize_parts),
        const_delinearize: make_delinearize(&const_delinearize_parts),
        max_len,
    }
}

impl StructInput {
    fn build_linearize(&self, input: &Input) -> FullyLinearized {
        let b0 = Ident::new("B0", Span::mixed_site());
        let b1 = Ident::new("B1", Span::mixed_site());
        let PartialLinearized {
            linearize,
            delinearize,
            const_linearize,
            const_delinearize,
            max_len,
        } = build_linearize_struct(input, &self.fields, &b0);
        let max_len = max_len.unwrap_or_else(|| quote!(1usize));
        let mut consts = vec![];
        consts.push(quote! { const B0: usize = 0; });
        consts.push(quote! { const B1: usize = #max_len; });
        FullyLinearized {
            linearize,
            delinearize: quote! { Self #delinearize },
            const_linearize,
            const_delinearize: quote! { Self #const_delinearize },
            max_len: b1.clone(),
            consts,
            const_names: vec![b0, b1],
        }
    }
}

impl EnumInput {
    fn build_linearize(&self, input: &Input) -> FullyLinearized {
        let mut linearize_cases = vec![];
        let mut delinearize_cases = vec![];
        let mut const_linearize_cases = vec![];
        let mut const_delinearize_cases = vec![];
        let mut consts = vec![];
        consts.push(quote! { const B0: usize = 0; });
        let mut prev_const_name = Ident::new("B0", Span::mixed_site());
        let mut const_base = prev_const_name.clone();
        let mut const_base_offset = 0;
        let mut const_names = vec![prev_const_name.clone()];
        for (variant_idx, variant) in self.variants.iter().enumerate() {
            let mut exposition = vec![];
            for (idx, field) in variant.fields.iter().enumerate() {
                let idx = LitInt::new(&idx.to_string(), Span::call_site());
                let generated_name = field.generated_name.as_ref().unwrap();
                match &field.original_name {
                    None => exposition.push(quote! { #idx: #generated_name }),
                    Some(i) => exposition.push(quote! { #i: #generated_name }),
                }
            }
            let exposition = quote! {
                { #(#exposition),* }
            };
            let PartialLinearized {
                linearize,
                delinearize,
                const_linearize,
                const_delinearize,
                max_len,
            } = build_linearize_struct(input, &variant.fields, &prev_const_name);
            let next_base = match &max_len {
                Some(len) => quote! { <Self as __C>::#prev_const_name + #len },
                None => {
                    const_base_offset += 1;
                    let offset = Literal::usize_unsuffixed(const_base_offset);
                    quote! { <Self as __C>::#const_base + #offset }
                }
            };
            let ident = &variant.ident;
            linearize_cases.push(quote! {
                Self::#ident #exposition => {
                    #linearize
                }
            });
            if input.attributes.enable_const {
                const_linearize_cases.push(quote! {
                    Self::#ident #exposition => {
                        #const_linearize
                    }
                });
            }
            let const_name = Ident::new(&format!("B{}", variant_idx + 1), Span::mixed_site());
            consts.push(quote! { const #const_name: usize = #next_base; });
            if variant.fields.is_empty() {
                let guard = if input.generics.params.is_empty() {
                    quote! {
                        <Self as __C>::#prev_const_name
                    }
                } else {
                    quote! {
                        n if n == <Self as __C>::#prev_const_name
                    }
                };
                delinearize_cases.push(quote! {
                    #guard => Self::#ident { },
                });
                if input.attributes.enable_const {
                    const_delinearize_cases.push(quote! {
                        #guard => Self::#ident { },
                    });
                }
            } else {
                let make_case = |delinearize: &TokenStream| {
                    quote! {
                        #[allow(clippy::impossible_comparisons)]
                        n if n >= <Self as __C>::#prev_const_name && n < <Self as __C>::#const_name => {
                            let linear = linear.wrapping_sub(<Self as __C>::#prev_const_name);
                            Self::#ident #delinearize
                        },
                    }
                };
                delinearize_cases.push(make_case(&delinearize));
                if input.attributes.enable_const {
                    const_delinearize_cases.push(make_case(&const_delinearize));
                }
            }
            prev_const_name = const_name;
            const_names.push(prev_const_name.clone());
            if max_len.is_some() {
                const_base = prev_const_name.clone();
                const_base_offset = 0;
            }
        }
        let make_linearize = |cases: &[TokenStream]| {
            if self.variants.is_empty() {
                quote! {
                    #[cold]
                    const fn unreachable() -> ! {
                        unsafe { core::hint::unreachable_unchecked() }
                    }
                    unreachable()
                }
            } else {
                quote! {
                    match self {
                        #(#cases)*
                    }
                }
            }
        };
        let make_delinearize = |cases: &[TokenStream]| {
            quote! {
                match linear {
                    #(#cases)*
                    _ => {
                        #[cold]
                        const fn unreachable() -> ! {
                            unsafe { core::hint::unreachable_unchecked() }
                        }
                        unreachable()
                    },
                }
            }
        };
        FullyLinearized {
            linearize: make_linearize(&linearize_cases),
            const_linearize: make_linearize(&const_linearize_cases),
            delinearize: make_delinearize(&delinearize_cases),
            const_delinearize: make_delinearize(&const_delinearize_cases),
            max_len: prev_const_name,
            const_names,
            consts,
        }
    }
}

impl Input {
    fn parse_enum(input: ItemEnum) -> syn::Result<Self> {
        let span = input.span();
        let mut critical_types = Vec::new();
        let mut variants = vec![];
        let mut i = 0;
        for variant in input.variants {
            let mut fields = vec![];
            for field in variant.fields {
                critical_types.push(field.ty.clone());
                let name = Ident::new(&format!("f{i}"), Span::mixed_site());
                i += 1;
                fields.push(StructField {
                    original_name: field.ident,
                    generated_name: Some(name),
                    ty: field.ty,
                })
            }
            variants.push(EnumVariant {
                ident: variant.ident,
                fields,
            });
        }
        Ok(Self {
            span,
            ident: input.ident,
            generics: input.generics,
            critical_types,
            kind: Kind::Enum(EnumInput { variants }),
            attributes: parse_attributes(&input.attrs)?,
        })
    }

    fn parse_struct(input: ItemStruct) -> syn::Result<Self> {
        let span = input.span();
        let mut critical_types = Vec::new();
        let mut fields = vec![];
        for field in input.fields {
            critical_types.push(field.ty.clone());
            fields.push(StructField {
                original_name: field.ident,
                generated_name: None,
                ty: field.ty,
            });
        }
        Ok(Self {
            span,
            ident: input.ident,
            generics: input.generics,
            critical_types,
            kind: Kind::Struct(StructInput { fields }),
            attributes: parse_attributes(&input.attrs)?,
        })
    }

    fn build_linearize(&self) -> FullyLinearized {
        match &self.kind {
            Kind::Struct(s) => s.build_linearize(self),
            Kind::Enum(e) => e.build_linearize(self),
        }
    }
}

fn parse_attributes(attrs: &[Attribute]) -> syn::Result<InputAttributes> {
    let mut res = InputAttributesOpt::default();
    for attr in attrs {
        if !attr.meta.path().is_ident("linearize") {
            continue;
        }
        let new: InputAttributesOpt = attr.meta.require_list()?.parse_args()?;
        res.enable_const |= new.enable_const;
        macro_rules! opt {
            ($name:ident) => {
                if new.$name.is_some() {
                    res.$name = new.$name;
                }
            };
        }
        opt!(crate_name);
    }
    Ok(InputAttributes {
        crate_name: res.crate_name.unwrap_or_else(|| parse_quote!(::linearize)),
        enable_const: res.enable_const,
    })
}

impl Parse for InputAttributesOpt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut res = Self::default();
        while !input.is_empty() {
            let key: TokenTree = input.parse()?;
            match key.to_string().as_str() {
                "crate" => {
                    let _: Token![=] = input.parse()?;
                    let path: Path = input.parse()?;
                    res.crate_name = Some(path);
                }
                "const" => {
                    res.enable_const = true;
                }
                _ => {
                    return Err(Error::new(
                        key.span(),
                        format!("Unknown attribute: {}", key),
                    ))
                }
            }
            if !input.is_empty() {
                let _: Token![,] = input.parse()?;
            }
        }
        Ok(res)
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: Item = input.parse()?;
        match item {
            Item::Enum(e) => Self::parse_enum(e),
            Item::Struct(s) => Self::parse_struct(s),
            _ => Err(Error::new(item.span(), "expected enum or struct")),
        }
    }
}
