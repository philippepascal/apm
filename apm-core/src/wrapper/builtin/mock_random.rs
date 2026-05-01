use crate::config::resolve_outcome;
use super::{load_transitions_with_outcomes, is_impl_mode, happy_script, sad_script, seed_from_ctx, write_and_spawn_script};
use crate::wrapper::{Wrapper, WrapperContext};

pub struct MockRandomWrapper;

impl Wrapper for MockRandomWrapper {
    fn spawn(&self, ctx: &WrapperContext) -> anyhow::Result<std::process::Child> {
        let transitions = load_transitions_with_outcomes(ctx)?;
        if transitions.is_empty() {
            anyhow::bail!(
                "mock-random: no valid transitions from state '{}'",
                ctx.current_state
            );
        }
        let seed = seed_from_ctx(ctx);
        let idx = (seed as usize) % transitions.len();
        let chosen = &transitions[idx];
        let outcome = resolve_outcome(&chosen.0, &chosen.1);
        let target = chosen.0.to.clone();
        let script = if outcome == "success" {
            happy_script(&ctx.ticket_id, &target, is_impl_mode(&transitions))
        } else {
            sad_script(&ctx.ticket_id, &target)
        };
        write_and_spawn_script("random", &script, ctx)
    }
}
