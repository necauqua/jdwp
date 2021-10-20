use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Data, Fields, Index, LitInt, Token, Type,
};

#[proc_macro_derive(JdwpReadable)]
pub fn jdwp_readable(item: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(item as syn::DeriveInput);

    if let Data::Struct(struct_data) = &derive_input.data {
        let ident = derive_input.ident;
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
            impl ::jdwp::codec::JdwpReadable for #ident {
                fn read<R: Read>(read: &mut R) -> ::std::io::Result<Self> {
                    #read
                }
            }
        };
        tokens.into()
    } else {
        panic!("Can derive only for structs")
    }
}

#[proc_macro_derive(JdwpWritable)]
pub fn jdwp_writable(item: TokenStream) -> TokenStream {
    let derive_input = syn::parse_macro_input!(item as syn::DeriveInput);

    if let Data::Struct(struct_data) = &derive_input.data {
        let ident = derive_input.ident;
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
            impl ::jdwp::codec::JdwpWritable for #ident {
                fn write<W: Write>(&self, write: &mut W) -> ::std::io::Result<()> {
                    #write
                    Ok(())
                }
            }
        };
        tokens.into()
    } else {
        panic!("Can derive only for structs")
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
        let command_id = input.parse()?;
        Ok(ShortCommandAttr {
            command_set,
            command_id,
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
        Ok(syntax_tree) => syntax_tree,
        Err(err) => return err.to_compile_error().into(),
    };

    let ident = &item.ident;

    let tokens = quote! {
        #item

        impl ::jdwp::commands::Command for #ident {
            const ID: ::jdwp::CommandId = CommandId::new(#command_set, #command_id);
            type Output = #reply_type;
        }
    };
    tokens.into()
}
