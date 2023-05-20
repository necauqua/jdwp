use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Data, Error, Fields, Index, ItemStruct, LitInt, PathArguments, Token, Type,
};

#[proc_macro_derive(JdwpReadable, attributes(skip))]
pub fn jdwp_readable(item: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(item as syn::DeriveInput);

    match &derive_input.data {
        Data::Struct(struct_data) => {
            let ident = derive_input.ident;
            let generic_params = derive_input.generics.params;
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
                impl<#generic_params> ::jdwp::codec::JdwpReadable for #ident<#generic_params> #generics_where {
                    fn read<R: ::std::io::Read>(read: &mut ::jdwp::codec::JdwpReader<R>) -> ::std::io::Result<Self> {
                        #read
                    }
                }
            };
            tokens.into()
        }
        Data::Enum(enum_data) => Error::new(
            enum_data.enum_token.span,
            "Can derive JdwpReadable only for structs",
        )
        .to_compile_error()
        .into(),
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
                impl<#generic_params> ::jdwp::codec::JdwpWritable for #ident<#generic_params> #generics_where {
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

    let new = if item.fields.is_empty() {
        quote!()
    } else {
        let mut typed_idents = Vec::with_capacity(item.fields.len());
        let mut idents = Vec::with_capacity(item.fields.len());
        for f in &item.fields {
            match f.ident {
                Some(ref ident) => {
                    let ty = &f.ty;

                    // this is very cringe but also very simple
                    let string_magic = quote!(#ty).to_string() == "String";

                    typed_idents.push(if string_magic {
                        quote!(#ident: impl Into<String>)
                    } else {
                        quote!(#ident: #ty)
                    });
                    idents.push(if string_magic {
                        quote!(#ident: #ident.into())
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
        let of = try_generate_of_constructor(&item);
        quote! {
            impl #ident {
                /// Autogenerated constructor to create the command
                pub fn new(#(#typed_idents,)*) -> Self {
                    Self { #(#idents,)* }
                }
                #of
            }
        }
    };

    let tokens = quote! {
        #item

        #new

        impl ::jdwp::commands::Command for #ident {
            const ID: ::jdwp::CommandId = ::jdwp::CommandId::new(#command_set, #command_id);
            type Output = #reply_type;
        }
    };
    tokens.into()
}

fn try_generate_of_constructor(item: &ItemStruct) -> proc_macro2::TokenStream {
    let field = &match item.fields {
        Fields::Named(ref named) => &named.named[0],
        Fields::Unnamed(ref unnamed) => &unnamed.unnamed[0],
        _ => unreachable!(),
    };

    match &field.ty {
        Type::Path(tp) => {
            if tp.path.segments.len() == 1 {
                let first = &tp.path.segments[0];
                let vec = &first.ident;
                if vec.to_string() == "Vec" {
                    if let PathArguments::AngleBracketed(args) = &first.arguments {
                        let f_ident = &field.ident;
                        let tpe = &args.args[0];
                        return quote! {
                            /// Autogenerated shortcut to create a command with a single value in the list
                            pub fn of(single: #tpe) -> Self {
                                Self { #f_ident: vec![single] }
                            }
                        };
                    }
                }
            }
        }
        _ => {}
    }

    quote!()
}
