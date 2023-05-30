use anyhow::Result;

mod msg;
mod state;

#[cfg(test)]
mod tests;

pub fn run(prefix: &str, alert_triggers: &[u64]) -> Result<()> {
    crate::pipeline::run_to_stdout(
        msg::Messages::from_run()?,
        state::State::new(prefix, alert_triggers)?,
    )
}
