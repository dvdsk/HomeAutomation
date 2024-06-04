use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
// use fasthash::{murmur3, Murmur3Hasher, SeaHasher};
use heapless::LinearMap;
/// static global error fmt message cache
use protocol::large_bedroom::bed::Error;

static ERROR_CACHE: Mutex<CriticalSectionRawMutex, LinearMap<u32, heapless::String<200>, 20>> =
    Mutex::new(LinearMap::new());

pub fn make_error_string(e: impl core::fmt::Debug) -> heapless::String<200> {
    defmt::warn!("hi");
    // let hash = murmur3::hash32(e);
    // let res = ERROR_CACHE.lock(|map| {
    //     map.get(&hash)
    // });
    heapless::String::new()
}
