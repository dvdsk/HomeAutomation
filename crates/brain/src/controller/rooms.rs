pub mod common;
pub mod entrance;
pub mod kitchen;
pub mod large_bedroom;
pub mod small_bedroom;

macro_rules! impl_open_or_wipe {
    ($store:ty) => {
        /// Maybe the struct has changed so this can no longer be opened.
        /// Just wipe it in that case, too lazy to do migrations.
        fn open_or_wipe(
            tree: sled::Tree,
        ) -> Result<$store, color_eyre::Report> {
            use ::color_eyre::eyre::WrapErr;

            if let Ok(store) = <$store>::open_tree(tree.clone()) {
                Ok(store)
            } else {
                tracing::error!(
                    "Could not open {}, wiping it and opening new one",
                    ::std::stringify!($store)
                );
                tree.clear().wrap_err("Error wiping tree")?;
                <$store>::open_tree(tree)
                    .wrap_err("Failed to open tree after wiping")
            }
        }
    };
}

use impl_open_or_wipe;
