use nestrs_core::module;

use crate::users::service::UsersService;

// Resolvers are not providers — they are declared once in `#[graphql]`
// (see `crate::graphql`), which composes the schema and attaches their
// discovery metadata.
#[module(providers = [UsersService])]
pub struct UsersModule;

#[cfg(test)]
mod tests {
    use super::*;
    use nestrs_core::{Container, Module};
    use std::sync::Arc;

    #[test]
    fn registers_users_service() {
        let container = UsersModule::register(Container::builder()).build();
        let svc: Option<Arc<UsersService>> = container.get();
        assert!(svc.is_some());
    }
}
