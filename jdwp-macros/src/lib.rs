use std::collections::HashSet;

use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Data, Error, Fields, GenericParam, Index, LitInt, Meta, Token, Type, TypePath,
};

fn get_generic_names(generic_params: &Punctuated<GenericParam, Comma>) -> proc_macro2::TokenStream {
    use GenericParam::*;

    let generics = generic_params.iter().map(|param| match param {
        Type(type_param) => type_param.ident.to_token_stream(),
        Lifetime(lifetime_def) => lifetime_def.lifetime.to_token_stream(),
        Const(const_param) => const_param.ident.to_token_stream(),
    });
    quote!(#(#generics,)*)
}

fn is_attr_ident(attr: &Attribute, name: &str) -> bool {
    match &attr.meta {
        Meta::List(list) => list.path.is_ident(name),
        Meta::NameValue(name_value) => name_value.path.is_ident(name),
        _ => false,
    }
}

fn is_generic_param(
    generic_params: &Punctuated<GenericParam, Comma>,
    ty: &Type,
) -> Option<TypePath> {
    if let Type::Path(path) = ty {
        if let Some(prefix) = path.path.segments.first() {
            return generic_params
                .iter()
                .any(|param| match param {
                    GenericParam::Type(type_param) => type_param.ident == prefix.ident,
                    _ => false,
                })
                .then(|| path.clone());
        }
    }
    None
}

#[proc_macro_derive(JdwpReadable, attributes(skip))]
pub fn jdwp_readable(item: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(item as syn::DeriveInput);

    match &derive_input.data {
        Data::Struct(struct_data) => {
            let generic_params = derive_input.generics.params;
            let mut field_types = HashSet::new();
            let read = match &struct_data.fields {
                Fields::Unit => quote!(Ok(Self)),
                Fields::Named(named) => {
                    let fields = named.named.iter().map(|f| {
                        let name = f.ident.as_ref().unwrap(); // we are in Named branch so this is not None
                        if let Some(path) = is_generic_param(&generic_params, &f.ty) {
                            field_types.insert(path);
                        }
                        quote!(#name: ::jdwp::codec::JdwpReadable::read(read)?)
                    });
                    quote!(Ok(Self { #(#fields),* }))
                }
                Fields::Unnamed(unnamed) => {
                    let fields = unnamed.unnamed.iter().map(|f| {
                        if let Some(path) = is_generic_param(&generic_params, &f.ty) {
                            field_types.insert(path);
                        }
                        quote!(::jdwp::codec::JdwpReadable::read(read)?)
                    });
                    quote!(Ok(Self(#(#fields),*)))
                }
            };

            let mut generic_predicates = quote!();
            if let Some(existing) = derive_input.generics.where_clause {
                for predicate in existing.predicates {
                    generic_predicates.extend(quote!(#predicate,));
                }
            }
            for field_type in field_types {
                generic_predicates.extend(quote!(#field_type: ::jdwp::codec::JdwpReadable,));
            }

            let ident = derive_input.ident;
            let generic_names = get_generic_names(&generic_params);
            let tokens = quote! {
                impl<#generic_params> ::jdwp::codec::JdwpReadable for #ident<#generic_names> where #generic_predicates {
                    fn read<R: ::std::io::Read>(read: &mut ::jdwp::codec::JdwpReader<R>) -> ::std::io::Result<Self> {
                        #read
                    }
                }
            };
            tokens.into()
        }
        Data::Enum(enum_data) => {
            let Some(repr) = derive_input
                .attrs
                .iter()
                .find(|attr| is_attr_ident(attr, "repr")) else {
                return Error::new(enum_data.enum_token.span, "No explicit repr")
                    .to_compile_error()
                    .into();
            };
            let repr = repr.parse_args::<Type>().expect("Bad repr"); // todo better error

            let mut match_arms = Vec::with_capacity(enum_data.variants.len());
            let mut field_types = HashSet::new();

            let generic_params = derive_input.generics.params;

            for v in &enum_data.variants {
                let Some((_, ref d)) = v.discriminant else {
                    return Error::new(v.span(), "No explicit discriminant")
                        .to_compile_error()
                        .into()
                };
                let name = &v.ident;

                let constructor = match &v.fields {
                    Fields::Named(named) => {
                        let fields = named.named.iter().map(|f| {
                            if let Some(path) = is_generic_param(&generic_params, &f.ty) {
                                field_types.insert(path);
                            }
                            f.ident.as_ref().unwrap()
                        });
                        quote!( { #(#fields: ::jdwp::codec::JdwpReadable::read(read)?,)* } )
                    }
                    Fields::Unnamed(unnamed) => {
                        let fields = unnamed.unnamed.iter().map(|f| {
                            if let Some(path) = is_generic_param(&generic_params, &f.ty) {
                                field_types.insert(path);
                            }
                            quote!(::jdwp::codec::JdwpReadable::read(read)?)
                        });
                        quote!( ( #(#fields),* ) )
                    }
                    Fields::Unit => quote!(),
                };
                match_arms.push(quote!(x if x == (#d) => Self::#name #constructor));
            }

            let mut generic_predicates = quote!();
            if let Some(existing) = derive_input.generics.where_clause {
                for predicate in existing.predicates {
                    generic_predicates.extend(quote!(#predicate,));
                }
            }
            for field_type in field_types {
                generic_predicates.extend(quote!(#field_type: ::jdwp::codec::JdwpReadable,));
            }

            let ident = derive_input.ident;
            let generic_names = get_generic_names(&generic_params);
            let tokens = quote! {
                impl<#generic_params> ::jdwp::codec::JdwpReadable for #ident<#generic_names> where #generic_predicates {
                    fn read<R: ::std::io::Read>(read: &mut ::jdwp::codec::JdwpReader<R>) -> ::std::io::Result<Self> {
                        let res = match #repr::read(read)? {
                            #(#match_arms,)*
                            _ => return Err(::std::io::Error::from(::std::io::ErrorKind::InvalidData)),
                        };
                        Ok(res)
                    }
                }
            };
            tokens.into()
        }
        Data::Union(union_data) => Error::new(
            union_data.union_token.span,
            "Can derive JdwpReadable only for structs and enums with explicit discriminants",
        )
        .to_compile_error()
        .into(),
    }
}

#[proc_macro_derive(JdwpWritable)]
pub fn jdwp_writable(item: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(item as syn::DeriveInput);

    match &derive_input.data {
        Data::Struct(struct_data) => {
            let write = match &struct_data.fields {
                Fields::Unit => quote!(),
                Fields::Named(named) => {
                    let fields = named.named.iter().map(|f| {
                        let name = f.ident.as_ref().unwrap(); // same as above here
                        quote!(self.#name.write(write)?)
                    });
                    quote!(#(#fields;)*)
                }
                Fields::Unnamed(unnamed) => {
                    let fields = (0..unnamed.unnamed.len()).map(|i| {
                        let idx = Index::from(i);
                        quote!(self.#idx.write(write)?)
                    });
                    quote!(#(#fields;)*)
                }
            };
            let ident = derive_input.ident;
            let generic_params = derive_input.generics.params;
            let generic_names = get_generic_names(&generic_params);
            let generics_where = derive_input.generics.where_clause;
            let tokens = quote! {
                impl<#generic_params> ::jdwp::codec::JdwpWritable for #ident<#generic_names> #generics_where {
                    fn write<W: ::std::io::Write>(&self, write: &mut ::jdwp::codec::JdwpWriter<W>) -> ::std::io::Result<()> {
                        #write
                        Ok(())
                    }
                }
            };
            tokens.into()
        }
        Data::Enum(enum_data) => {
            let Some(repr) = derive_input
                .attrs
                .iter()
                .find(|attr| is_attr_ident(attr, "repr")) else {
                return Error::new(enum_data.enum_token.span, "No explicit repr")
                    .to_compile_error()
                    .into();
            };
            let repr = repr.parse_args::<Type>().expect("TODO");

            let mut match_arms = Vec::with_capacity(enum_data.variants.len());

            for v in &enum_data.variants {
                let Some((_, ref d)) = v.discriminant else {
                    return Error::new(
                        v.span(),
                        "No explicit discriminant",
                    )
                    .to_compile_error()
                    .into()
                };

                let (destruct, writes) = match &v.fields {
                    Fields::Named(named) => {
                        let names = named
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().unwrap())
                            .collect::<Vec<_>>();
                        (quote!({ #(#names),* }), quote!(#(#names.write(write)?;)*))
                    }
                    Fields::Unnamed(unnamed) => {
                        let names = (0..unnamed.unnamed.len())
                            .map(|i| Ident::new(&format!("case_{i}"), unnamed.span()))
                            .collect::<Vec<_>>();
                        (quote!((#(#names),*)), quote!(#(#names.write(write)?;)*))
                    }
                    Fields::Unit => (quote!(), quote!()),
                };

                let name = &v.ident;

                match_arms.push(quote! {
                    Self::#name #destruct => {
                        #repr::write(&(#d), write)?;
                        #writes
                    }
                });
            }
            let ident = derive_input.ident;
            let generic_params = derive_input.generics.params;
            let generic_names = get_generic_names(&generic_params);
            let generics_where = derive_input.generics.where_clause;
            let tokens = quote! {
                impl<#generic_params> ::jdwp::codec::JdwpWritable for #ident<#generic_names> #generics_where {
                    fn write<W: ::std::io::Write>(&self, write: &mut ::jdwp::codec::JdwpWriter<W>) -> ::std::io::Result<()> {
                        match self {
                            #(#match_arms)*
                        }
                        Ok(())
                    }
                }
            };
            tokens.into()
        }
        Data::Union(union_data) => Error::new(
            union_data.union_token.span,
            "Can derive JdwpWritable only for structs and enums with explicit discriminants",
        )
        .to_compile_error()
        .into(),
    }
}

struct CommandAttr {
    reply_type: Type,
    command: ShortCommandAttr,
}

impl Parse for CommandAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let reply_type = input.parse()?;
        _ = input.parse::<Token![,]>()?;
        Ok(CommandAttr {
            reply_type,
            command: input.parse()?,
        })
    }
}

struct ShortCommandAttr {
    command_set: LitInt,
    command_id: LitInt,
}

impl ShortCommandAttr {
    fn long(self, reply_type: Type) -> CommandAttr {
        CommandAttr {
            reply_type,
            command: self,
        }
    }
}

impl Parse for ShortCommandAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let command_set = input.parse()?;
        _ = input.parse::<Token![,]>()?;
        Ok(ShortCommandAttr {
            command_set,
            command_id: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn jdwp_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemStruct);

    let attr = syn::parse::<CommandAttr>(attr.clone()).or_else(|_| {
        syn::parse::<ShortCommandAttr>(attr)
            .and_then(|sca| syn::parse_str(&format!("{}Reply", item.ident)).map(|t| sca.long(t)))
    });
    let CommandAttr {
        reply_type,
        command: ShortCommandAttr {
            command_set,
            command_id,
        },
    } = match attr {
        Ok(attr) => attr,
        Err(err) => return err.to_compile_error().into(),
    };

    let ident = &item.ident;
    let generic_params = &item.generics.params;
    let generic_names = get_generic_names(generic_params);
    let generics_where = &item.generics.where_clause;

    let new = if !item.fields.is_empty() {
        let mut docs = Vec::with_capacity(item.fields.len());
        let mut typed_idents = Vec::with_capacity(item.fields.len());
        let mut idents = Vec::with_capacity(item.fields.len());
        for f in &item.fields {
            match f.ident {
                Some(ref ident) => {
                    let ty = &f.ty;

                    // this is pretty cringe but eh
                    let phantom = quote!(#ty).to_string().starts_with("PhantomData ");

                    if !phantom {
                        typed_idents.push(quote!(#ident: #ty));
                    }

                    docs.push(
                        f.attrs
                            .iter()
                            .find(|attr| is_attr_ident(attr, "doc"))
                            .map(|attr| {
                                quote! {
                                    #[doc = stringify!(#ident)]
                                    #[doc = " - "]
                                    #attr
                                    #[doc = "\n"]
                                }
                            }),
                    );

                    idents.push(if phantom {
                        quote!(#ident: ::std::marker::PhantomData)
                    } else {
                        quote!(#ident)
                    });
                }
                None => {
                    return Error::new(item.fields.span(), "Command struct must use named fields")
                        .to_compile_error()
                        .into()
                }
            }
        }
        quote! {
            impl<#generic_params> #ident<#generic_names> #generics_where {
                /// Autogenerated constructor to create the command
                /// ### Arguments:
                #(#docs)*
                pub fn new(#(#typed_idents,)*) -> Self {
                    Self { #(#idents,)* }
                }
            }
        }
    } else {
        quote!()
    };

    let tokens = quote! {
        #item
        #new

        impl<#generic_params> ::jdwp::spec::Command for #ident<#generic_names> #generics_where {
            const ID: ::jdwp::spec::CommandId = ::jdwp::spec::CommandId::new(#command_set, #command_id);
            type Output = #reply_type;
        }
    };
    tokens.into()
}
