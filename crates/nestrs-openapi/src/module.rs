//! `OpenApiModule` — import it to serve the auto-generated OpenAPI document and
//! Swagger UI over HTTP.

use nestrs_core::{ContainerBuilder, DynamicModule, Module};
use nestrs_http::HttpEndpointMeta;
use poem::{get, Route};

use crate::document::build_document;
use crate::ui;

// NestJS convention (`SwaggerModule.setup('api', …)`): UI at `/api`, document
// at `/api-json`. The OpenAPI spec mandates no serving path, so we follow the
// reference NestJS surface this framework mirrors. The bundled Swagger UI
// references these paths absolutely, so they are fixed (not yet configurable).
const DOCS_PATH: &str = "/api";
const SPEC_PATH: &str = "/api-json";

/// Document metadata for the generated OpenAPI spec — the `info` block. Pass it
/// via [`OpenApiModule::for_root`] to override the defaults.
#[derive(Clone, Debug)]
pub struct OpenApiOptions {
    /// `info.title`.
    pub title: String,
    /// `info.version`.
    pub version: String,
    /// `info.description`, omitted when `None`.
    pub description: Option<String>,
}

impl Default for OpenApiOptions {
    fn default() -> Self {
        Self {
            title: "nestrs API".into(),
            version: "0.1.0".into(),
            description: None,
        }
    }
}

/// Add to a `#[module(imports = [...])]` to expose:
/// - `GET /api-json` — the OpenAPI 3.1 document, and
/// - `GET /api` — bundled Swagger UI.
///
/// Like [`nestrs_graphql::GraphqlModule`], it self-mounts via an
/// [`HttpEndpointMeta`]: there is nothing to wire in `main.rs`. The spec is
/// composed from every `#[controller]` linked into the binary, so importing
/// this module is the only step.
///
/// Imported **by type** it uses [`OpenApiOptions::default`]:
///
/// ```ignore
/// #[module(imports = [OpenApiModule])]
/// ```
///
/// Imported **via [`for_root`](OpenApiModule::for_root)** it takes document
/// metadata — the analog of NestJS's `SwaggerModule.setup(...)`:
///
/// ```ignore
/// #[module(imports = [
///     OpenApiModule::for_root(OpenApiOptions {
///         title: "My API".into(),
///         version: "1.0".into(),
///         ..Default::default()
///     }),
/// ])]
/// ```
pub struct OpenApiModule;

impl OpenApiModule {
    /// Configure the document metadata at the import site. Returns a
    /// [`DynamicModule`] to list in `#[module(imports = [...])]`.
    pub fn for_root(options: OpenApiOptions) -> OpenApiSetup {
        OpenApiSetup { options }
    }
}

impl Module for OpenApiModule {
    fn register(builder: ContainerBuilder) -> ContainerBuilder {
        register(builder, OpenApiOptions::default())
    }
}

/// The configured form of [`OpenApiModule`], produced by
/// [`OpenApiModule::for_root`].
pub struct OpenApiSetup {
    options: OpenApiOptions,
}

impl DynamicModule for OpenApiSetup {
    fn register(self, builder: ContainerBuilder) -> ContainerBuilder {
        register(builder, self.options)
    }
}

/// Shared registration for both the default and configured paths: install the
/// self-mounting endpoint, capturing the document metadata in the mount closure
/// (the document itself is built once at `configure`, when every controller is
/// present).
fn register(builder: ContainerBuilder, options: OpenApiOptions) -> ContainerBuilder {
    builder.provide_meta(HttpEndpointMeta::new(
        DOCS_PATH,
        "openapi",
        move |container, route: Route| {
            let document = build_document(
                container,
                &options.title,
                &options.version,
                options.description.as_deref(),
            );
            let spec =
                serde_json::to_string_pretty(&document).unwrap_or_else(|_| document.to_string());
            route
                .at(SPEC_PATH, get(ui::spec_endpoint(spec)))
                .at(DOCS_PATH, get(ui::swagger_index))
                .at("/api/swagger-ui.css", get(ui::swagger_css))
                .at("/api/swagger-ui-bundle.js", get(ui::swagger_bundle))
                .at(
                    "/api/swagger-ui-standalone-preset.js",
                    get(ui::swagger_preset),
                )
        },
    ))
}
