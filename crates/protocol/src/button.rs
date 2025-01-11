#[cfg(feature = "alloc")]
use crate::reading::LabelFormatter;
use serde::{Deserialize, Serialize};

// Duration of the press in milliseconds
#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    postcard::experimental::max_size::MaxSize,
    defmt::Format,
    PartialEq,
    Eq,
)]
pub struct Press(pub u16);

impl Press {
    #[must_use]
    pub fn is_long(&self) -> bool {
        self.0 > 300
    }
}

/// turns an enum of empty variants into one with [`Press`](crate::Press) inside each variant also
/// adds a method `press(&self)` that returns an instance of [`Press`](crate::Press). It can be
/// used to quickly find out if a button event is a long or short press.
macro_rules! button_enum {
    (
        $(#[$outer:meta])*
        $name:ident {$($variant:ident,)*}
        $device:expr;
    ) => {
        #[derive(strum::VariantNames)]
        #[derive(
            Clone,
			Copy,
			Debug,
			serde::Serialize,
			serde::Deserialize,
            defmt::Format,
            postcard::experimental::max_size::MaxSize,
            PartialEq,
            Eq,
        )]
        $(#[$outer])* // docs
        /// SAFETY: must be repr(u8) or id `fn` will create undefined behavior
        #[repr(u8)]
        pub enum $name {
            $($variant(crate::button::Press),)*
        }

        impl $name {
            #[must_use]
            pub fn press(&self) -> crate::button::Press {
                match self {
                    $(Self::$variant(d) => *d,)*
                }
            }

            pub fn variant_name(&self) -> &'static str {
                match self {
                    $(Self::$variant(_) => stringify!($variant),)*
                }
            }
        }

        impl From<$name> for f32 {
            fn from(val: $name) -> Self {
                match val {
                    $($name::$variant(p) if p.is_long() => 2.0,)*
                    $($name::$variant(_) => 1.0,)*
                }
            }
        }

        #[cfg(feature = "alloc")]
        impl crate::reading::tree::Tree for $name {
            fn inner(&self) -> crate::reading::tree::Item<'_> {
                crate::reading::tree::Item::Leaf(crate::reading::Info {
                    val: (*self).into(),
                    device: $device,
                    description: "button",
                    range: 0.0..3.0,
                    resolution: 1.0,
                    unit: crate::Unit::None,
                    branch_id: self.branch_id(),
                    label_formatter: Box::new(crate::button::ButtonLabelFormatter),
                })
            }
            fn name(&self) -> String {
                self.variant_name().to_owned()
            }
            fn branch_id(&self) -> crate::reading::tree::Id {
                // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a
                // `repr(C)` `union` between `repr(C)` structs, each of which
                // has the `u8` discriminant as its first field, so we can read
                // the discriminant without offsetting the pointer.
                let discriminant = unsafe { *<*const _>::from(self).cast::<u8>() };
                discriminant as crate::reading::tree::Id
            }
        }

        impl crate::IsSameAs for $name {
            #[must_use]
            fn is_same_as(&self, other: &Self) -> bool {
                match (self, other) {
                    $(($name::$variant(_), $name::$variant(_)) => true,)*
                    (_, _) => false,
                }
            }
        }
    };
}
pub(crate) use button_enum;

#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct ButtonLabelFormatter;

#[cfg(feature = "alloc")]
impl LabelFormatter for ButtonLabelFormatter {
    fn format(&self, info: &crate::reading::Info) -> String {
        match info.val {
            2.0 => "long press",
            1.0 => "short press",
            0.0 => "not pressed",
            _ => "ButtonLabelFormatter error",
        }
        .to_string()
    }
    fn box_clone(&self) -> Box<dyn LabelFormatter> {
        Box::new(Self)
    }
}
