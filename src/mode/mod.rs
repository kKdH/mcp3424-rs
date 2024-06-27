pub use continuous::ContinuousMode;
pub use multishot::MultiShotMode;
pub use oneshot::OneShotMode;

mod continuous;
mod multishot;
mod oneshot;

pub trait Mode {}
