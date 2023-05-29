use notify_rust::{Notification, Urgency};

pub enum Level {
    Lo,
    Mid,
    Hi,
}

pub struct Alert {
    notification: Notification,
}

impl Alert {
    pub fn new(level: Level, threshold: u64, current: u64) -> Self {
        let mut notification = Notification::new();
        notification
            .summary(&format!("Battery power bellow {}%!", threshold))
            .body(&format!("{}%", current))
            .urgency(match level {
                Level::Lo => Urgency::Low,
                Level::Mid => Urgency::Normal,
                Level::Hi => Urgency::Critical,
            });
        Self { notification }
    }
}

impl crate::Alert for Alert {
    fn send(&self) -> anyhow::Result<()> {
        self.notification.show()?;
        Ok(())
    }
}
