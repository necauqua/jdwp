use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Data, Error, Fields, GenericParam, Index, LitInt, Token, Type,
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

#[proc_macro_derive(JdwpReadable, attributes(skip))]
pub fn jdwp_readable(item: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(item as syn::DeriveInput);

    match &derive_input.data {
        Data::Struct(struct_data) => {
            let ident = derive_input.ident;
            let generic_params = derive_input.generics.params;
            let generic_names = get_generic_names(&generic_params);
            let generics_where = derive_input.generics.where_clause;
            let read = match &struct_data.fields {
                Fields::Unit => quote!(Ok(Self)),
                Fields::Named(named) => {
                    let fields = named.named.iter().map(|f| {
                        let name = f.ident.as_ref().unwrap(); // we are in Named branch so this is not None
                        quote!(#name: ::jdwp::codec::JdwpReadable::read(read)?)
                    });
                    quote!(Ok(Self { #(#fields),* }))
                }
                Fields::Unnamed(unnamed) => {
                    let fields = (0..unnamed.unnamed.len())
                        .map(|_| quote!(::jdwp::codec::JdwpReadable::read(read)?));
                    quote!(Ok(Self(#(#fields),*)))
                }
            };
            let tokens = quote! {
                impl<#generic_params> ::jdwp::codec::JdwpReadable for #ident<#generic_names> #generics_where {
                    fn read<R: ::std::io::Read>(read: &mut ::jdwp::codec::JdwpReader<R>) -> ::std::io::Result<Self> {
                        #read
                    }
                }
            };
            tokens.into()
        }
        Data::Enum(enum_data) => {
            let fields = enum_data
                .variants
                .iter()
                .map(|variant| match variant.fields {
                    Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                        Some((&variant.ident, &fields.unnamed[0].ty))
                    }
                    _ => None,
                })
                .collect::<Option<Vec<_>>>();

            if let Some(fields) = fields {
                let ident = derive_input.ident;

                let mut locals = Vec::with_capacity(fields.len());
                let mut match_arms = Vec::with_capacity(fields.len());

                for (i, (variant, ty)) in fields.iter().enumerate() {
                    locals.push(quote! {
                        <::core::option::Option::<#ty> as ::jdwp::codec::JdwpReadable>::read(read)?
                    });
                    let mut match_arm = Vec::with_capacity(fields.len());
                    for j in 0..fields.len() {
                        match_arm.push(if j == i {
                            quote!(::core::option::Option::Some(matched))
                        } else {
                            quote!(::core::option::Option::None)
                        })
                    }
                    match_arms.push(quote! {
                        (#(#match_arm),*) => ::std::result::Result::Ok(#ident::#variant(matched))
                    });
                }

                let tokens = quote! {
                    impl ::jdwp::codec::JdwpReadable for #ident {
                        fn read<R: ::std::io::Read>(read: &mut ::jdwp::codec::JdwpReader<R>) -> ::std::io::Result<Self> {
                            match (#(#locals),*) {
                                #(#match_arms,)*
                                _ => ::std::result::Result::Err(::std::io::Error::new(::std::io::ErrorKind::InvalidData, "Multiple values in response")),
                            }
                        }
                    }
                };
                tokens.into()
            } else {
                Error::new(
                        enum_data.enum_token.span,
                        "Can derive JdwpReadable only for enums with all variants having a single unnamed field",
                    )
                    .to_compile_error()
                    .into()
            }
        }
        Data::Union(union_data) => Error::new(
            union_data.union_token.span,
            "Can derive JdwpReadable only for structs",
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
            let ident = derive_input.ident;
            let generic_params = derive_input.generics.params;
            let generic_names = get_generic_names(&generic_params);
            let generics_where = derive_input.generics.where_clause;

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
        Data::Enum(enum_data) => Error::new(
            enum_data.enum_token.span,
            "Can derive JdwpWritable only for structs",
        )
        .to_compile_error()
        .into(),
        Data::Union(union_data) => Error::new(
            union_data.union_token.span,
            "Can derive JdwpWritable only for structs",
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
        let _ = input.parse::<Token![,]>()?;
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
        let _ = input.parse::<Token![,]>()?;
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

    let new = if item.fields.is_empty() {
        quote!()
    } else {
        let mut docs = Vec::with_capacity(item.fields.len());
        let mut typed_idents = Vec::with_capacity(item.fields.len());
        let mut idents = Vec::with_capacity(item.fields.len());
        for f in &item.fields {
            match f.ident {
                Some(ref ident) => {
                    let ty = &f.ty;

                    // this is very cringe but also very simple
                    let stype = quote!(#ty).to_string();
                    let string_magic = stype == "String";
                    let phantom = stype.starts_with("PhantomData ");

                    if !phantom {
                        typed_idents.push(if string_magic {
                            quote!(#ident: impl Into<String>)
                        } else {
                            quote!(#ident: #ty)
                        });
                    }

                    docs.push(f.attrs.iter().find(|a| a.path.is_ident("doc")).map(|a| {
                        let tokens = &a.tokens;
                        quote! {
                            #[doc = stringify!(#ident)]
                            #[doc = " - "]
                            #[doc #tokens]
                            #[doc = "\n"]
                        }
                    }));

                    idents.push(if string_magic {
                        quote!(#ident: #ident.into())
                    } else if phantom {
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
    };

    let tokens = quote! {
        #item
        #new

        impl<#generic_params> ::jdwp::commands::Command for #ident<#generic_names> #generics_where {
            const ID: ::jdwp::CommandId = ::jdwp::CommandId::new(#command_set, #command_id);
            type Output = #reply_type;
        }
    };
    tokens.into()
}
