use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::reading::tree::{self, Tree};
use crate::{pir, IsSameAs};

crate::button_enum! {
    /// +----------------------------------------------------+
    /// | LL | LM | LR  <desk controls>  RLm | RL | RR | RRm |
    /// +----------------------------------------------------+
    ///
    /// LL   LeftLeft,
    /// LN   LeftMiddle,
    /// LR   LeftRight,
    /// RLm  RightLeftmost,
    /// RL   RightLeft,
    /// RR   RightRight,
    /// RRm  RightRightmost,
    Button {
        LeftLeft,
        LeftMiddle,
        LeftRight,
        RightLeftmost,
        RightLeft,
        RightRight,
        RightRightmost,
    }
    crate::Device::LargeBedroom(super::Device::DeskRight(Device));
}

#[derive(
    strum::EnumDiscriminants,
    Clone,
    Copy,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    PartialEq,
)]
#[strum_discriminants(derive(Hash))]
#[repr(u8)]
pub enum Reading {
    Button(Button),
    Pir(pir::Status),
}

#[cfg(feature = "alloc")]
impl Tree for Reading {
    fn inner(&self) -> tree::Item<'_> {
        match self {
            Reading::Button(button_panel) => button_panel.inner(),
            Reading::Pir(status) => {
                use crate::reading::Info;

                let info = Info {
                    val: *status as u8 as f32,
                    device: Device.rooted(),
                    description: "Pir motion detector",
                    range: 0.0..=2.0,
                    resolution: 1.0,
                    unit: crate::Unit::None,
                    branch_id: self.branch_id(),
                    label_formatter: Box::new(crate::pir::PirLabelFormatter),
                };

                tree::Item::Leaf(info)
            }
        }
    }

    fn inner_mut(&mut self) -> tree::ItemMut<'_> {
        use crate::reading::tree::field_as_any;

        let value = field_as_any!(self, Button, Pir);
        tree::ItemMut::Leaf(value)
    }

    fn branch_id(&self) -> tree::Id {
        ReadingDiscriminants::from(self) as tree::Id
    }
}

impl IsSameAs for Reading {
    fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Button(a), Self::Button(b)) => a.is_same_as(b),
            (Self::Pir(_), Self::Pir(_)) => true,
            (_, _) => false,
        }
    }
}

#[derive(
    Clone,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    Eq,
    PartialEq,
    Hash,
)]
pub struct Device;

impl Device {
    #[must_use]
    pub fn rooted(self) -> crate::Device {
        crate::Device::LargeBedroom(crate::large_bedroom::Device::DeskRight(
            self,
        ))
    }

    #[must_use]
    pub const fn info(&self) -> crate::DeviceInfo {
        macro_rules! rtree {
            [$($reading:expr),+] => {
                [$(crate::Reading::LargeBedroom(
                    crate::large_bedroom::Reading::DeskRight($reading),
                )),+]
            };
        }

        crate::DeviceInfo {
            name: "Desk Buttons and Pir",
            affects_readings: &rtree![
                Reading::Button(Button::LeftLeft(crate::button::Press(0))),
                Reading::Pir(crate::pir::Status::Unknown)
            ],
            ..crate::DeviceInfo::button_defaults()
        }
    }
}
