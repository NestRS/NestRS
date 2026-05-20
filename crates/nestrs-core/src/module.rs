use crate::container::ContainerBuilder;

pub trait Module {
    fn register(builder: ContainerBuilder) -> ContainerBuilder;
}
