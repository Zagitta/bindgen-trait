#[cfg(feature = "guest")]
mod guest;
#[cfg(feature = "host")]
mod host;

#[cfg(feature = "guest")]
pub use guest::{Host, MyGuestInterface};

#[cfg(feature = "host")]
pub use host::{MyGuestInterface, MyHostInterface, Runtime};
