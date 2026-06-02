use nestrs_config::ConfigModule;
use nestrs_core::module;

use crate::oauth::config::IssuerConfig;
use crate::oauth::service::{OAuthFlow, TokenIssuer};
use crate::oauth::strategy::{
    ClientAuthGuard, ClientCredentialsStrategy, OAuthGuard, OAuthStrategy,
};
use crate::users::UsersModule;

#[module(
    imports = [ConfigModule::for_feature::<IssuerConfig>(), UsersModule],
    providers = [
        TokenIssuer,
        OAuthFlow,
        OAuthStrategy,
        OAuthGuard,
        ClientCredentialsStrategy,
        ClientAuthGuard,
    ],
)]
pub struct OAuthModule;
