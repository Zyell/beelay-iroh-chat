/// invoke bindings with some modifications, and impl_trait are taken from https://github.com/jvatic/tauri-ipc-macros
/// Event macros differ completely, they are applied to structs to have been type parsing from taur-sys listen
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::parse::ParseStream;
use syn::{
    self, AngleBracketedGenericArguments, Field, FieldMutability, FnArg,
    GenericArgument, Ident, ItemFn, ItemTrait, LitStr, Pat, PathArguments, Signature,
    Token, TraitItem, Type, TypePath, Visibility, braced,
    parse::Parse,
    parse_macro_input, parse_quote,
    punctuated::{Pair, Punctuated},
    token::{self, Comma},
};

#[derive(Default)]
struct InvokeBindingAttrs {
    cmd_prefix: Option<String>,
}

impl Parse for InvokeBindingAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs: Self = Default::default();
        while !input.is_empty() {
            let kv: KeyValuePair = input.parse()?;
            if kv.key.as_str() == "cmd_prefix" {
                attrs.cmd_prefix = Some(kv.value)
            }
        }
        Ok(attrs)
    }
}

struct KeyValuePair {
    key: String,
    value: String,
}

impl Parse for KeyValuePair {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let value: LitStr = input.parse()?;
        Ok(Self {
            key: key.to_string(),
            value: value.value(),
        })
    }
}

fn extract_result_types(ty: &Type) -> Option<(&Type, &Type)> {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let segment = path.segments.last()?;
            if segment.ident != "Result" {
                return None;
            }

            match &segment.arguments {
                PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
                    if args.len() == 2 {
                        if let (GenericArgument::Type(ok_type), GenericArgument::Type(err_type)) =
                            (&args[0], &args[1])
                        {
                            Some((ok_type, err_type))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Apply this to a trait, and generate an implementation for it's fns in the
/// same scope that call `invoke` using the fn name as the command
///
/// # Examples
///
/// ```ignore
/// #[allow(async_fn_in_trait)]
/// #[tauri_bindgen_rs_macros::invoke_bindings]
/// pub trait Commands {
///     async hello(name: String) -> Result<String, String>;
/// }
///
/// async fn hello_world() -> Result<String, String> {
///     hello("world".into())
/// }
/// ```
#[proc_macro_attribute]
pub fn invoke_bindings(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as InvokeBindingAttrs);
    let trait_item = parse_macro_input!(tokens as ItemTrait);
    let fn_items = trait_item.items.iter().fold(Vec::new(), |mut m, item| {
        if let TraitItem::Fn(fn_item) = item {
            let fields: Punctuated<Field, Token![,]> =
                Punctuated::from_iter(fn_item.sig.inputs.iter().fold(Vec::new(), |mut m, arg| {
                    let pt = match arg {
                        FnArg::Typed(pt) => pt,
                        FnArg::Receiver(_) => {
                            panic!("receiver arguments not supported");
                        }
                    };
                    let ident = match pt.pat.as_ref() {
                        Pat::Ident(pi) => Some(pi.ident.clone()),
                        _ => panic!("argument not supported"),
                    };
                    let colon_token = Some(pt.colon_token);
                    let ty = pt.ty.as_ref().clone();
                    m.push(Field {
                        attrs: Vec::new(),
                        vis: Visibility::Inherited,
                        mutability: FieldMutability::None,
                        ident,
                        colon_token,
                        ty,
                    });
                    m
                }));
            let field_names: Punctuated<Ident, Token![,]> =
                Punctuated::from_iter(fields.iter().map(|field| field.ident.clone().unwrap()));
            let fn_name = fn_item.sig.ident.to_string();
            let fn_name = attrs
                .cmd_prefix
                .clone()
                .map_or(fn_name.clone(), |prefix| prefix + fn_name.as_str());
            let invocation = match fn_item.sig.output {
                syn::ReturnType::Default => {
                    quote! { ::tauri_sys::core::invoke::<()> }
                }
                syn::ReturnType::Type(_, ref ty) => {
                    if let Some((v, e)) = extract_result_types(ty) {
                        quote! { ::tauri_sys::core::invoke_result::<#v, #e> }
                    } else {
                        quote! { ::tauri_sys::core::invoke::<#ty> }
                    }
                }
            };
            m.push(ItemFn {
                attrs: Vec::new(),
                vis: trait_item.vis.clone(),
                sig: fn_item.sig.clone(),
                block: parse_quote!({
                    #[derive(::serde::Serialize)]
                    #[serde(rename_all = "camelCase")]
                    struct Args {
                        #fields
                    }
                    let args = Args { #field_names };
                    #invocation(#fn_name, args).await
                }),
            });
        }
        m
    });
    let fn_items = ItemList { list: fn_items };
    let mod_visibility = trait_item.vis.clone();
    let ret = quote! {
        #trait_item
        #mod_visibility mod ui{
            use super::*;
            #fn_items
        }
    };

    TokenStream::from(ret)
}

struct ImplTrait {
    trait_ident: Ident,
    fns: ItemList<ItemFn>,
}

impl Parse for ImplTrait {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fns;
        let trait_ident = input.parse()?;
        let _: Token![,] = input.parse()?;
        let _: token::Brace = braced!(fns in input);
        let fns = fns.parse()?;
        Ok(ImplTrait { trait_ident, fns })
    }
}

struct ItemList<I: ToTokens> {
    list: Vec<I>,
}

impl<I: Parse + ToTokens> Parse for ItemList<I> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut list = Vec::new();

        while !input.is_empty() {
            let item: I = input.parse()?;
            list.push(item);
        }

        Ok(ItemList { list })
    }
}

impl<I: ToTokens> ToTokens for ItemList<I> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(self.list.iter());
    }
}

/// Takes the name of a trait and an impl block, and emits a ghost struct that
/// implements that trait using the provided fn signatures—stripping away any
/// generics and arguments with `tauri` as the first path segment.
///
/// TODO: accept a list of arguments to ignore vs relying on the `tauri::` prefix.
///
/// # Examples
///
/// ```ignore
/// trait Commands {
///     async foo(bar: String) -> Result<(), String>;
///     async bar(foo: String) -> Result<(), String>;
/// }
///
/// // ignore this (here so the example can compile)
/// mod tauri {
///     struct State {}
/// }
///
/// tauri_bindgen_rs_macros::impl_trait!(Commands, {
///     // we'll also need a #[tauri::command] attribute here
///     async foo(state: tauri::State, bar: String) -> Result<(), String> {
///         Ok(())
///     }
///
///     // we'll also need a #[tauri::command] attribute here
///     async bar(state: tauri::State, foo: String) -> Result<(), String> {
///         Ok(())
///     }
/// });
/// ```
#[proc_macro]
pub fn impl_trait(tokens: TokenStream) -> TokenStream {
    let ImplTrait { trait_ident, fns } = parse_macro_input!(tokens as ImplTrait);

    let mut trait_fns = Vec::new();

    fn map_fn_input(mut item: Pair<FnArg, Comma>) -> Pair<FnArg, Comma> {
        let value = item.value_mut();
        if let FnArg::Typed(pt) = value {
            if let Pat::Ident(pi) = pt.pat.as_mut() {
                pi.ident = Ident::new(
                    // add an _ prefix to all fn arguments so we don't trigger unused variable warnings
                    { "_".to_string() + pi.ident.to_string().as_str() }.as_str(),
                    pi.ident.span(),
                );
            }
        }
        item
    }

    fn filter_map_fn_inputs(inputs: Punctuated<FnArg, Comma>) -> Punctuated<FnArg, Comma> {
        let tauri_ident = Ident::new("tauri", Span::call_site());
        Punctuated::from_iter(inputs.into_pairs().fold(Vec::new(), |mut m, item| {
            if let Some(tp) = match item.value() {
                FnArg::Typed(pt) => match pt.ty.as_ref() {
                    Type::Path(path) => Some(path),
                    _ => None,
                },
                _ => None,
            } {
                if let Some(s) = tp.path.segments.first() {
                    if s.ident == tauri_ident {
                        return m;
                    }
                }
            }
            m.push(map_fn_input(item));
            m
        }))
    }

    fns.list.iter().for_each(|func| {
        let sig = &func.sig;
        trait_fns.push(ItemFn {
            attrs: Vec::new(),
            vis: func.vis.clone(),
            sig: Signature {
                constness: None,
                asyncness: sig.asyncness,
                unsafety: None,
                abi: None,
                fn_token: sig.fn_token,
                generics: Default::default(),
                ident: sig.ident.clone(),
                paren_token: sig.paren_token,
                inputs: filter_map_fn_inputs(sig.inputs.clone()),
                variadic: None,
                output: sig.output.clone(),
            },
            block: parse_quote!({ todo!() }),
        });
    });

    let fn_listing = trait_fns
        .iter()
        .map(|fn_item| fn_item.sig.ident.clone())
        .collect::<Vec<Ident>>();
    let tauri_command_handler = quote! {
        pub fn command_handler<R>() -> impl Fn(::tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
            where
                R: ::tauri::Runtime,
            {
                ::tauri::generate_handler![
                    #(#fn_listing),*
                ]
            }
    };

    let struct_name = Ident::new(format!("__Impl{}", trait_ident).as_str(), Span::call_site());
    let trait_fns = ItemList { list: trait_fns };

    let ret = quote! {
        struct #struct_name {}

        impl #trait_ident for #struct_name {
            #trait_fns
        }

        #fns

        #tauri_command_handler
    };

    TokenStream::from(ret)
}

struct EventDefinition {
    name: LitStr,
    payload_type: Type,
}

impl Parse for EventDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let name: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;
        let payload_type: Type = content.parse()?;
        Ok(EventDefinition { name, payload_type })
    }
}

struct EventMacroInput {
    ui_attrs: proc_macro2::TokenStream,
    tauri_attrs: proc_macro2::TokenStream,
    events: Vec<EventDefinition>,
}

impl Parse for EventMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse "ui"
        let ui = input.parse::<Ident>()?; // "ui"
        if ui.to_string() != "ui" {
            panic!("invalid attribute, expected `ui`");
        }
        input.parse::<Token![=]>()?;

        // Parse ui attributes as a token stream until we hit a comma
        let mut ui_attrs = proc_macro2::TokenStream::new();
        while !input.peek(Token![,]) {
            let token: proc_macro2::TokenTree = input.parse()?;
            ui_attrs.extend(std::iter::once(token));
        }
        input.parse::<Token![,]>()?;

        // Parse "tauri"
        let tauri = input.parse::<Ident>()?; // "tauri"
        input.parse::<Token![=]>()?;
        if tauri.to_string() != "tauri" {
            panic!("invalid attribute, expected `tauri`");
        }

        // Parse tauri attributes as a token stream until we hit a comma
        let mut tauri_attrs = proc_macro2::TokenStream::new();
        while !input.peek(Token![,]) {
            let token: proc_macro2::TokenTree = input.parse()?;
            tauri_attrs.extend(std::iter::once(token));
        }
        input.parse::<Token![,]>()?;

        // Parse events block
        let content;
        syn::braced!(content in input);

        let mut events = Vec::new();
        while !content.is_empty() {
            events.push(content.parse::<EventDefinition>()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(EventMacroInput {
            ui_attrs,
            tauri_attrs,
            events,
        })
    }
}

#[proc_macro]
pub fn derive_events(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EventMacroInput);

    let ui_attrs = &input.ui_attrs;
    let tauri_attrs = &input.tauri_attrs;

    let mut ui_structs = Vec::new();
    let mut tauri_structs = Vec::new();

    for event in &input.events {
        let event_name_str = &event.name;
        let event_name_ident = Ident::new(&event.name.value(), event.name.span());
        let payload_type = &event.payload_type;

        // Generate UI struct
        tauri_structs.push(quote! {
            pub struct #event_name_ident(pub #payload_type);
            
            impl #event_name_ident {
                pub fn new(value: #payload_type) -> Self {
                    Self(value)
                }
                
                pub fn event_name() -> &'static str {
                    #event_name_str
                }
                
                pub fn emit<R: ::tauri::Runtime>(self, handle: & ::tauri::AppHandle<R>) -> ::core::result::Result<(), ::tauri::Error> {
                    let topic = Self::event_name().to_string();
                    handle.emit(&topic, self.0)
                }
            }
        });

        // Generate Tauri struct
        ui_structs.push(quote! {
            pub struct #event_name_ident(pub #payload_type);
            
            impl #event_name_ident {
                pub fn new(value: #payload_type) -> Self {
                    Self(value)
                }
                
                pub fn event_name() -> &'static str {
                    #event_name_str
                }
                
                pub async fn listen() -> ::core::result::Result<impl ::futures_core::Stream<Item = ::tauri_sys::event::Event<#payload_type>>, ::tauri_sys::Error> {
                    ::tauri_sys::event::listen::<#payload_type>(Self::event_name()).await
                }
            }
        });
    }

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        pub mod events {
            use super::*;

            #tauri_attrs
            pub mod tauri {
                use super::*;
                use ::tauri::Emitter;
                #(#tauri_structs)*
            }

            #ui_attrs
            pub mod ui {
                use super::*;
                #(#ui_structs)*
            }
        }
    };

    TokenStream::from(expanded)
}
