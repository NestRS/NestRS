use crate::authn::strategy::AppJwtStrategy;

pub type AuthGuard = nestrs_auth::AuthGuard<AppJwtStrategy>;
