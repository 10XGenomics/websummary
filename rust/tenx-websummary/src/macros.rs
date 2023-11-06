/// Shortcut to implement ReactComponent
#[macro_export]
macro_rules! react_component {
    ($struct_name:ident, $comp_name:literal) => {
        impl ReactComponent for $struct_name {
            fn component_name() -> &'static str {
                $comp_name
            }
        }
    };
}
