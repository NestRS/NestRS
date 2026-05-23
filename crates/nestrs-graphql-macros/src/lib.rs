//! GraphQL decorator macros, re-exported by `nestrs-graphql`. The generated
//! code uses absolute paths (`::nestrs_graphql::*`, `::nestrs_http::*`,
//! `::poem::*`, `::nestrs_core::*`), so this crate does not depend on them —
//! they resolve at the call site.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, parse_macro_input, Ident, ItemStruct, LitStr, Path, Token};

use nestrs_macro_support::{build_injectable_body, from_container_method, InjectableBody};

// -----------------------------------------------------------------------------
// #[resolver]
// -----------------------------------------------------------------------------

/// Mark a struct as a GraphQL resolver.
///
/// Behaves like `#[injectable]` for construction only (fields with `#[inject]`
/// resolved from the container, others default) — it emits `from_container`
/// and nothing else. A resolver is **not** a provider and is **not**
/// `Discoverable`: it is named once in `#[graphql(queries | mutations |
/// subscriptions = [...])]`, which both composes the static
/// `Schema<Q, M, S>` and attaches the per-resolver `GraphQLResolverMeta`
/// (the list it appears in *is* its kind). Listing a resolver in a
/// `#[module]`'s `providers` is therefore a compile error — the single
/// source of truth is the `#[graphql]`.
#[proc_macro_attribute]
pub fn resolver(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    if !args.is_empty() {
        return syn::Error::new_spanned(
            &args,
            "#[resolver] takes no arguments; a resolver's kind comes from the \
             `#[graphql(queries | mutations | subscriptions = [...])]` list it appears in",
        )
        .to_compile_error()
        .into();
    }

    let mut item = parse_macro_input!(input as ItemStruct);

    let InjectableBody { ctor, .. } = match build_injectable_body(&mut item) {
        Ok(body) => body,
        Err(err) => return err.to_compile_error().into(),
    };

    let name = item.ident.clone();
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let from_container = from_container_method(&ctor);

    quote! {
        #item

        impl #impl_generics #name #ty_generics #where_clause {
            #from_container
        }
    }
    .into()
}

// -----------------------------------------------------------------------------
// #[graphql(queries = [...], mutations = [...], subscriptions = [...])]
// -----------------------------------------------------------------------------

/// Compose discovered resolvers into a single GraphQL schema.
///
/// Applied to a unit marker struct. The macro generates one
/// `async_graphql::MergedObject` per root (Query / Mutation /
/// Subscription) and an inherent `build(container) -> Schema<...>`
/// method that constructs each resolver via `from_container` and
/// assembles the schema. The container is also attached as schema data.
///
/// Composition is static because `async-graphql`'s root types live in
/// the `Schema<Q, M, S>` parameters — they cannot be assembled
/// dynamically at runtime from a `DiscoveryService` walk.
///
/// `queries = [...]` is required (a GraphQL schema needs a Query root).
/// `mutations` and `subscriptions` are optional — when omitted or empty,
/// async-graphql's `EmptyMutation` / `EmptySubscription` are used.
///
/// The marker struct is always `Discoverable`, so it is listed in a
/// `#[module]`'s `providers`. Its registration attaches one
/// `GraphQLResolverMeta` per listed resolver (the list it appears in is its
/// kind) — this is the single place resolvers are declared.
///
/// `path = "/graphql"` is optional. When set, registration *also* mounts the
/// schema over HTTP — `POST <path>` (queries) and `GET <path>` (a generated
/// playground) via an `HttpEndpointMeta` — so no `.mount()` call is needed in
/// `main.rs`. When omitted, the app mounts `build`'s schema by hand.
#[proc_macro_attribute]
pub fn graphql(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as GraphQlArgs);
    let item = parse_macro_input!(input as ItemStruct);
    let name = item.ident.clone();

    if args.queries.is_empty() {
        return syn::Error::new_spanned(
            &item,
            "#[graphql] requires a non-empty `queries = [...]` list",
        )
        .to_compile_error()
        .into();
    }

    let query_root = format_ident!("__{}QueryRoot", name);
    let mutation_root = format_ident!("__{}MutationRoot", name);
    let subscription_root = format_ident!("__{}SubscriptionRoot", name);

    let queries = &args.queries;
    let mutations = &args.mutations;
    let subscriptions = &args.subscriptions;

    let query_decl = quote! {
        #[derive(::nestrs_graphql::async_graphql::MergedObject)]
        pub struct #query_root(#(pub #queries),*);
    };
    let query_expr = quote! {
        #query_root(#(<#queries>::from_container(&container)),*)
    };

    let (mutation_decl, mutation_ty, mutation_expr) = if mutations.is_empty() {
        (
            quote!(),
            quote!(::nestrs_graphql::async_graphql::EmptyMutation),
            quote!(::nestrs_graphql::async_graphql::EmptyMutation),
        )
    } else {
        (
            quote! {
                #[derive(::nestrs_graphql::async_graphql::MergedObject)]
                pub struct #mutation_root(#(pub #mutations),*);
            },
            quote!(#mutation_root),
            quote! {
                #mutation_root(#(<#mutations>::from_container(&container)),*)
            },
        )
    };

    let (subscription_decl, subscription_ty, subscription_expr) = if subscriptions.is_empty() {
        (
            quote!(),
            quote!(::nestrs_graphql::async_graphql::EmptySubscription),
            quote!(::nestrs_graphql::async_graphql::EmptySubscription),
        )
    } else {
        (
            quote! {
                #[derive(::nestrs_graphql::async_graphql::MergedSubscription)]
                pub struct #subscription_root(#(pub #subscriptions),*);
            },
            quote!(#subscription_root),
            quote! {
                #subscription_root(#(<#subscriptions>::from_container(&container)),*)
            },
        )
    };

    // `#[graphql]` is the single source of truth for resolvers, so it
    // attaches one `GraphQLResolverMeta` per listed resolver — the list a
    // resolver appears in *is* its kind. `#[resolver]` no longer does this,
    // which is why resolvers are not (and cannot be) module providers.
    let meta_attach = |resolvers: &[Path], kind: TokenStream2| -> Vec<TokenStream2> {
        resolvers
            .iter()
            .map(|r| {
                quote! {
                    builder = builder.attach_meta::<#r, ::nestrs_graphql::GraphQLResolverMeta>(
                        ::nestrs_graphql::GraphQLResolverMeta::new(#kind),
                    );
                }
            })
            .collect()
    };
    let query_metas = meta_attach(queries, quote!(::nestrs_graphql::ResolverKind::Query));
    let mutation_metas = meta_attach(mutations, quote!(::nestrs_graphql::ResolverKind::Mutation));
    let subscription_metas =
        meta_attach(subscriptions, quote!(::nestrs_graphql::ResolverKind::Subscription));

    // When `path` is given, the schema also mounts itself over HTTP: a
    // playground handler plus an `HttpEndpointMeta` serving `POST <path>`
    // (queries) and `GET <path>` (playground), with no `.mount()` in `main.rs`.
    let (playground_fn, endpoint_attach) = match &args.path {
        None => (quote!(), quote!()),
        Some(path) => {
            let playground = format_ident!("__nestrs_gql_playground_{}", name);
            (
                quote! {
                    #[allow(non_snake_case)]
                    #[::poem::handler]
                    fn #playground() -> ::poem::web::Html<::std::string::String> {
                        ::poem::web::Html(
                            ::nestrs_graphql::async_graphql::http::playground_source(
                                ::nestrs_graphql::async_graphql::http::GraphQLPlaygroundConfig::new(#path),
                            ),
                        )
                    }
                },
                quote! {
                    builder = builder.attach_meta::<#name, ::nestrs_http::HttpEndpointMeta>(
                        ::nestrs_http::HttpEndpointMeta::new(#path, "graphql", |__c, __r| {
                            let __schema = <#name>::build(__c.clone());
                            __r.nest(
                                #path,
                                ::poem::post(
                                    ::nestrs_graphql::async_graphql_poem::GraphQL::new(__schema),
                                )
                                .get(#playground),
                            )
                        }),
                    );
                },
            )
        }
    };

    quote! {
        #item

        #query_decl
        #mutation_decl
        #subscription_decl

        impl #name {
            pub fn build(
                container: ::nestrs_core::Container,
            ) -> ::nestrs_graphql::async_graphql::Schema<
                #query_root,
                #mutation_ty,
                #subscription_ty,
            > {
                ::nestrs_graphql::async_graphql::Schema::build(
                    #query_expr,
                    #mutation_expr,
                    #subscription_expr,
                )
                .data(container)
                .finish()
            }
        }

        #playground_fn

        impl ::nestrs_core::Discoverable for #name {
            fn register(
                mut builder: ::nestrs_core::ContainerBuilder,
            ) -> ::nestrs_core::ContainerBuilder {
                #(#query_metas)*
                #(#mutation_metas)*
                #(#subscription_metas)*
                #endpoint_attach
                builder
            }
        }
    }
    .into()
}

#[derive(Default)]
struct GraphQlArgs {
    path: Option<LitStr>,
    queries: Vec<Path>,
    mutations: Vec<Path>,
    subscriptions: Vec<Path>,
}

impl Parse for GraphQlArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = GraphQlArgs::default();
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "path" => args.path = Some(input.parse()?),
                "queries" | "mutations" | "subscriptions" => {
                    let content;
                    bracketed!(content in input);
                    let paths: Punctuated<Path, Token![,]> =
                        Punctuated::parse_terminated(&content)?;
                    match key.to_string().as_str() {
                        "queries" => args.queries.extend(paths),
                        "mutations" => args.mutations.extend(paths),
                        "subscriptions" => args.subscriptions.extend(paths),
                        _ => unreachable!("guarded by the outer match arm"),
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[graphql] key `{other}` (expected `path`, `queries`, `mutations`, or `subscriptions`)"
                        ),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(args)
    }
}
