use crate::{Device, Reading, Unit};

#[derive(Debug)]
pub struct ReadingInfo {
    pub val: f32,
    pub device: Device,
    pub resolution: f32,
    pub range: core::ops::Range<f32>,
    pub unit: Unit,
    pub description: &'static str,
    pub branch_id: u8,
}

impl ReadingInfo {
    #[must_use]
    pub fn from_same_device(&self) -> &'static [Reading] {
        self.device.affected_readings()
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
    };
}

pub(crate) use all_nodes;
