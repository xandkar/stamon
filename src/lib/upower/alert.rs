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
    pub fn new(level: Level, summary: &str, body: &str) -> Self {
        let mut notification = Notification::new();
        notification
            .summary(summary)
            .body(body)
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
