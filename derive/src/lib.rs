use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, ItemFn};

/// Make a function support the `spawn!` macro.
///
/// # Fields
///
/// * Field `spawner` will be consumed to spawn bundles.
///
/// Accepts `&mut World`, `&mut Commands`, `&mut ChildBuilder`, `&mut WorldChildBuilder`.
///
/// * Field `asset_server` will be used to enable `@asset` and `@load`.
///
/// Accepts `AssetServer`.
#[proc_macro_attribute]
pub fn spawner_fn(_: TokenStream1, tokens: TokenStream1) -> TokenStream1 {
    spawner_fn2(tokens.into()).into()
}

fn spawner_fn2(tokens: TokenStream) -> TokenStream {
    let Ok(mut f) = syn::parse2::<ItemFn>(tokens.clone()) else {
        return quote! {#tokens compile_error!("Expected function.")};
    };

    let bevy_spawn_fn = match proc_macro_crate::crate_name("bevy_spawn_fn") {
        Ok(FoundCrate::Itself) => {
            quote! {crate}
        }
        Ok(FoundCrate::Name(name)) => format_ident!("{name}").into_token_stream(),
        Err(_) => return quote! {#tokens compile_error!("Expected crate bevy_spawn_fn.")},
    };

    let mut asset_server_found = false;

    for item in &f.sig.inputs {
        match item {
            syn::FnArg::Receiver(_) => (),
            syn::FnArg::Typed(pat_type) => {
                if syn::parse2::<Ident>(pat_type.pat.to_token_stream())
                    .is_ok_and(|x| x == format_ident!("asset_server"))
                {
                    asset_server_found = true
                }
            }
        }
    }

    let block = f.block;

    if asset_server_found {
        f.block = parse_quote!({
            #bevy_spawn_fn::spawner_scope(spawner, || {
                #bevy_spawn_fn::asset_server_scope(&asset_server, || #block)
            })
        });
    } else {
        f.block = parse_quote!({
            #bevy_spawn_fn::spawner_scope(spawner, || #block)
        });
    }

    f.to_token_stream()
}

/// Make a system function support the `spawn!` macro.
#[proc_macro_attribute]
pub fn spawner_system(_: TokenStream1, tokens: TokenStream1) -> TokenStream1 {
    spawner_system2(tokens.into()).into()
}

fn spawner_system2(tokens: TokenStream) -> TokenStream {
    let Ok(mut f) = syn::parse2::<ItemFn>(tokens.clone()) else {
        return quote! {#tokens compile_error!("Expected function.")};
    };

    let bevy_spawn_fn = match proc_macro_crate::crate_name("bevy_spawn_fn") {
        Ok(FoundCrate::Itself) => {
            quote! {crate}
        }
        Ok(FoundCrate::Name(name)) => format_ident!("{name}").into_token_stream(),
        Err(_) => return quote! {#tokens compile_error!("Expected crate bevy_spawn_fn.")},
    };

    f.sig.inputs.push(parse_quote!(
        mut __spawn_commands: #bevy_spawn_fn::Commands
    ));
    f.sig.inputs.push(parse_quote!(
        __spawn_asset_server: #bevy_spawn_fn::Res<#bevy_spawn_fn::AssetServer>
    ));

    let block = f.block;

    f.block = parse_quote!({
        #bevy_spawn_fn::spawner_scope(&mut __spawn_commands, || {
            #bevy_spawn_fn::asset_server_scope(&__spawn_asset_server, || #block)
        })
    });
    f.to_token_stream()
}
