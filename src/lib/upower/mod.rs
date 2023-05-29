use anyhow::Result;

mod alert;
mod msg;
mod state;

#[cfg(test)]
mod tests;

pub fn run(prefix: &str, alert_triggers: &[u64]) -> Result<()> {
    crate::pipeline_to_stdout(
        msg::Messages::from_run()?,
        state::State::new(prefix, alert_triggers)?,
    )
}
