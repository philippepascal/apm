use crate::config::resolve_outcome;
use super::{load_transitions_with_outcomes, is_impl_mode, happy_script, write_and_spawn_script};
use crate::wrapper::{Wrapper, WrapperContext};

pub struct MockHappyWrapper;

impl Wrapper for MockHappyWrapper {
    fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child> {
        let transitions = load_transitions_with_outcomes(ctx)?;
        let success: Vec<_> = transitions.iter()
            .filter(|(t, s)| resolve_outcome(t, s) == "success")
            .collect();
        match success.len() {
            0 => anyhow::bail!(
                "mock-happy: no success-outcome transition from state '{}'",
                ctx.current_state
            ),
            1 => {},
            n => anyhow::bail!(
                "mock-happy: {} success-outcome transitions found from state '{}'; expected exactly 1",
                n, ctx.current_state
            ),
        }
        let target = success[0].0.to.clone();
        let impl_mode = is_impl_mode(&transitions);
        let script = happy_script(&ctx.ticket_id, &target, impl_mode);
        write_and_spawn_script("happy", &script, ctx)
    }
}
