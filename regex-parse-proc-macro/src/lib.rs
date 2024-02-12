use darling::{ast::NestedMeta, Error, FromField, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Path, Type};

#[derive(FromField, Debug)]
#[darling(attributes(regex))]
struct RegexSubArgs {
    ident: Option<Ident>,
    ty: Type,
    method: Option<Ident>,
}

#[derive(FromMeta)]
struct RegexArgs {
    regex: String,
}

#[proc_macro_attribute]
pub fn regex(args: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(Error::from(e).write_errors()),
    };

    let mut ast = syn::parse(stream).expect("macro input stream should be valid");

    // Generate the arguments in the form of a struct
    let args = match RegexArgs::from_list(attr_args.as_ref()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    impl_macro(&args, &mut ast)
}

fn impl_macro(attrs: &RegexArgs, ast: &mut syn::ItemStruct) -> TokenStream {
    let result = ast.fields.clone().into_iter().map(|x| {
        let args = RegexSubArgs::from_field(&x).expect("field should be parsable");

        let ty = args.ty.clone();

        let ident = args.ident.clone();

        let ident_string = args
            .ident
            .clone()
            .expect("Ident should be some")
            .to_string();

        let code = match args.method {
            Some(t) => quote! {{
                let item = match matches.name(#ident_string) {
                    Some(item) => {
                        item.as_str()
                    },
                    None => return Err(format!("Item with name {} not found", #ident_string).into())
                };
                #t(item)?
            }},
            None => quote! {{
                type T = #ty;
                let item = match matches.name(#ident_string) {
                    Some(item) => {
                        item.as_str()
                    },
                    None => return Err(format!("Item with name {} not found", #ident_string).into())
                };
                T::parse(item)?
            }},
        };

        quote! {
            #ident: #code,
        }
    });

    let ident = ast.ident.clone();
    let regex = attrs.regex.clone();

    ast.fields.iter_mut().for_each(|x| {
        x.attrs
            .retain(|x| x.path() != &Path::from_string("regex").expect("regex should be path"))
    });

    quote! {
        #ast

        impl RegexParse for #ident {
            fn parse(text: &str) -> Result<Self, Box<dyn Error>> {
                let regex = Regex::new(#regex)?;

                let matches = regex.captures(text);

                match matches {
                    Some(matches) => Ok(Self{
                        #(#result)*
                    }),
                    None => {
                        Err("Failed to parse text for regex".into())
                    }
                }

            }
        }
    }
    .into()
}
