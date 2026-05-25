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

#[module(providers = [Orphan])]
struct BrokenModule;

#[test]
#[should_panic(expected = "cannot resolve providers")]
fn missing_dependency_panics_with_a_clear_message() {
    let _ = BrokenModule::register(Container::builder()).build();
}
