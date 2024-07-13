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
    pub fn is_long(&self) -> bool {
        if self.0 > 300 {
            true
        } else {
            false
        }
    }
}

/// turns an enum of empty variants into one with [`Press`](crate::Press) inside each variant also
/// adds a method `press(&self)` that returns an instance of [`Press`](crate::Press). It can be
/// used to quickly find out if a button event is a long or short press.
///
/// Example:
///
/// button_enum! {
///     DeskButton {
///         OneOfFour,
///         TwoOfFour,
///     }
/// }
///  
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
        /// SAFETY: must be repr(u8) or id fn will create undefined behaviour
        #[repr(u8)]
        pub enum $name {
            $($variant(crate::button::Press),)*
        }

        impl $name {
            pub fn press(&self) -> crate::button::Press {
                match self {
                    $(Self::$variant(d) => *d,)*
                }
            }
        }

        impl Into<f32> for $name {
            fn into(self) -> f32 {
                match self {
                    $(Self::$variant(p) if p.is_long() => 2.0,)*
                    $(Self::$variant(_) => 1.0,)*
                }
            }
        }

        #[cfg(feature = "alloc")]
        impl crate::reading_tree::Tree for $name {
            fn inner<'a>(&'a self) -> crate::reading_tree::Item<'a> {
                crate::reading_tree::Item::Leaf(crate::reading_tree::ReadingInfo {
                    val: (*self).into(),
                    device: $device,
                    description: "button",
                    range: 0.0..3.0,
                    resolution: 1.0,
                    unit: crate::Unit::None,
                    branch_id: self.branch_id(),
                })
            }
            fn name(&self) -> String {
                let dbg_repr = format!("{:?}", self);
                dbg_repr
                    .split_once('(')
                    .map(|(name, _)| name)
                    .unwrap_or("-")
                    .to_string()
            }
            fn branch_id(&self) -> crate::reading_tree::Id {
                // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a
                // `repr(C)` `union` between `repr(C)` structs, each of which
                // has the `u8` discriminant as its first field, so we can read
                // the discriminant without offsetting the pointer.
                let discriminant = unsafe { *<*const _>::from(self).cast::<u8>() };
                discriminant as crate::reading_tree::Id
            }
        }
    };
}
pub(crate) use button_enum;
