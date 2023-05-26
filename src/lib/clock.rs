use std::time::Duration;

pub type Tick = (); // TODO Make opaque. Need to update callers.

/// First tick is immediate, subsequent ones after the given interval.
pub fn new(interval: Duration) -> impl Iterator<Item = Tick> {
    std::iter::once(()).chain(Clock { interval })
}

struct Clock {
    interval: Duration,
}

impl Iterator for Clock {
    type Item = Tick;

    fn next(&mut self) -> Option<Self::Item> {
        std::thread::sleep(self.interval);
        Some(())
    }
}
