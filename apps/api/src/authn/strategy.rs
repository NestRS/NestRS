use identity::Claims;
use nestrs_auth::JwtStrategy;

pub type AppJwtStrategy = JwtStrategy<Claims>;
