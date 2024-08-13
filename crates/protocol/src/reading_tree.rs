use crate::{Device, Reading, Unit};

#[derive(Debug, Clone)]
pub struct ReadingInfo {
    pub val: f32,
    pub device: Device,
    /// smallest step size the data can make
    pub resolution: f32,
    pub range: core::ops::Range<f32>,
    pub unit: Unit,
    pub description: &'static str,
    pub branch_id: u8,
}

impl ReadingInfo {
    #[must_use]
    pub fn from_same_device(&self) -> &'static [Reading] {
        self.device.info().affects_readings
    }

    /// useful for printing/formatting floats
    /// # Example
    /// ```rust
    /// use crate::Reading;
    /// use crate::large_bedroom;
    /// use crate::large_bedroom::desk;
    ///
    /// let reading =
    /// Reading::LargeBedroom(large_bedroom::Reading(desk::Reading::Temperature(22.428124);
    ///
    /// let info = reading.leaf();
    /// let printed = format!("{0:.1$}", info.val, info.precision());
    /// assert_eq!(printed, "22.42");
    /// ```
    pub fn precision(&self) -> usize {
        if self.resolution > 1.0 {
            0
        } else {
            self.resolution.log10().abs() as usize
        }
    }
}

pub type Id = u8;
#[derive(Debug)]
pub enum Item<'a> {
    Leaf(ReadingInfo),
    Node(&'a dyn Tree),
}

pub trait Tree: core::fmt::Debug {
    fn inner(&self) -> Item<'_>;
    fn leaf(&self) -> ReadingInfo
    where
        Self: Sized,
    {
        let mut current = self as &dyn Tree;
        loop {
            match current.inner() {
                Item::Node(node) => current = node,
                Item::Leaf(leaf) => return leaf,
            }
        }
    }
    fn name(&self) -> String {
        let dbg_repr = format!("{self:?}");
        dbg_repr
            .split_once('(')
            .map_or("_", |(name, _)| name)
            .to_string()
    }
    fn branch_id(&self) -> Id;
}

macro_rules! impl_zero {
    ($name:ident; $($var:ident),*) => {
        impl $name {
            #[must_use]
            pub fn is_same_as(&self, other: &Self) -> bool {
                match (self, other) {
                    $(($name::$var(a), $name::$var(b)) => a.is_same_as(b),)*
                    (_, _) => false,
                }
            }
        }

    };
}
pub(crate) use impl_zero;

macro_rules! all_nodes {
    ($name:ident; $variant:ident; $($var:ident),*) => {
        impl crate::reading_tree::Tree for $name {
            fn inner(&self) -> crate::reading_tree::Item<'_> {
                match self {
                    $(
                    $name::$var(inner) => crate::reading_tree::Item::Node(inner as &dyn crate::reading_tree::Tree)
                    ),*
                }
            }

            fn branch_id(&self) -> crate::reading_tree::Id {
                $variant::from(self) as crate::reading_tree::Id
            }

        }
        crate::reading_tree::impl_zero!{$name; $($var),*}
    };
}
pub(crate) use all_nodes;
