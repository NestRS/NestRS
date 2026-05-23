//! `#[module(providers = [...])]` registers in dependency order, not list
//! order. A provider may be listed before the dependency it injects, and a
//! genuinely missing dependency fails fast at boot with a readable message.

use std::sync::Arc;

use nestrs_core::{injectable, module, Container, Module};

#[injectable]
#[derive(Default)]
struct Dependency;

impl Dependency {
    fn value(&self) -> u32 {
        21
    }
}

#[injectable]
struct Consumer {
    #[inject]
    dep: Arc<Dependency>,
}

impl Consumer {
    fn doubled(&self) -> u32 {
        self.dep.value() * 2
    }
}

// `Consumer` is listed *before* the `Dependency` it injects.
#[module(providers = [Consumer, Dependency])]
struct ReversedModule;

#[test]
fn provider_listed_before_its_dependency_still_resolves() {
    let container = ReversedModule::register(Container::builder()).build();
    let consumer: Arc<Consumer> = container.get().expect("Consumer resolves");
    assert_eq!(consumer.doubled(), 42);
}

#[injectable]
struct Orphan {
    #[inject]
    _missing: Arc<Dependency>,
}

// `Dependency` is never provided, so `Orphan` can never be built.
#[module(providers = [Orphan])]
struct BrokenModule;

#[test]
#[should_panic(expected = "cannot resolve providers")]
fn missing_dependency_panics_with_a_clear_message() {
    let _ = BrokenModule::register(Container::builder()).build();
}
