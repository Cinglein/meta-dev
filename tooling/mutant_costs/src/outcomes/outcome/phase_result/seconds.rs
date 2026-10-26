use serde::Deserialize;

use task::Millis;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(transparent)]
pub struct Seconds(f64);

impl Seconds {
    const MILLIS_PER_SECOND: f64 = 1000.0;

    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a measured duration is finite and positive, and whole milliseconds are enough"
    )]
    pub const fn millis(self) -> Millis {
        Millis::new((self.0 * Self::MILLIS_PER_SECOND) as u64)
    }
}
