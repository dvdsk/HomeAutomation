pub mod radiator;

/// implements the `IsSameAs` trait for enum `thing` with variants `variant`
/// example usage:
/// ```rust
/// impl_is_same_as!(Reading; Temperature, Heating, SetBy, Setpoint);
/// ```
macro_rules! impl_is_same_as {
    ($thing:ident; $($variant:ident),+$(; $extra:pat => $todo:expr);*) => {
        impl crate::IsSameAs for $thing {
            fn is_same_as(&self, other: &Self) -> bool {
                match (self, other) {
                    $((Self::$variant(_), Self::$variant(_)))|+ => true,
                    $($extra => $todo,)*
                    _ => false,
                }
            }
        }
    };
}

pub(crate) use impl_is_same_as;
