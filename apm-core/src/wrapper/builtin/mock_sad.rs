use crate::config::resolve_outcome;
use super::{load_transitions_with_outcomes, sad_script, seed_from_ctx, write_and_spawn_script};
use crate::wrapper::{Wrapper, WrapperContext};

pub struct MockSadWrapper;

impl Wrapper for MockSadWrapper {
    fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child> {
        let transitions = load_transitions_with_outcomes(ctx)?;
        let eligible: Vec<_> = transitions.iter()
            .filter(|(t, s)| resolve_outcome(t, s) != "success")
            .collect();
        if eligible.is_empty() {
            anyhow::bail!(
                "mock-sad: no non-success transitions from state '{}'",
                ctx.current_state
            );
        }
        let seed = seed_from_ctx(ctx);
        let idx = (seed as usize) % eligible.len();
        let target = eligible[idx].0.to.clone();
        let script = sad_script(&ctx.ticket_id, &target);
        write_and_spawn_script("sad", &script, ctx)
    }
}
