use nestrs_core::module;
use nestrs_throttler::ThrottlerGuard;

use crate::oauth::controller::OAuthController;
use crate::oauth::service::TokenIssuer;
use crate::oauth::strategy::{OAuthGuard, OAuthStrategy};

#[module(providers = [
    TokenIssuer,
    OAuthStrategy,
    OAuthGuard,
    ThrottlerGuard,
    OAuthController,
])]
pub struct OAuthModule;
