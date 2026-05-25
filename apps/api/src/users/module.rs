use nestrs_core::module;

use crate::users::controller::UsersController;
use crate::users::service::UsersService;

#[module(providers = [UsersService, UsersController])]
pub struct UsersModule;

#[cfg(test)]
mod tests {
    use super::*;
    use nestrs_core::{Container, Module};
    use std::sync::Arc;

    #[test]
    fn registers_users_service() {
        let container = UsersModule::register(
            Container::builder().provide(sea_orm::DatabaseConnection::default()),
        )
        .build();
        let svc: Option<Arc<UsersService>> = container.get();
        assert!(svc.is_some());
    }
}
