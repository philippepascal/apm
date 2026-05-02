use crate::config::resolve_outcome;
use super::{load_transitions_with_outcomes, is_impl_mode, happy_script, sad_script, seed_from_ctx, write_and_spawn_script};
use crate::wrapper::{Wrapper, WrapperContext};

pub struct MockRandomWrapper;

pub(crate) fn pick_transition_idx(seed: u64, count: usize) -> usize {
    (seed as usize) % count
}

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
        let idx = pick_transition_idx(seed, transitions.len());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_transition_idx_is_deterministic_for_same_seed() {
        let count = 5;
        assert_eq!(pick_transition_idx(42, count), pick_transition_idx(42, count));
        assert_eq!(pick_transition_idx(42, count), 42 % 5);
    }

    #[test]
    fn pick_transition_idx_distributes_across_seeds() {
        // Sample 100 seeds, expect at least 3 distinct buckets across 5 transitions.
        let count = 5;
        let mut buckets = std::collections::HashSet::new();
        for seed in 0u64..100 {
            buckets.insert(pick_transition_idx(seed, count));
        }
        assert!(buckets.len() >= 3, "expected >=3 distinct outcomes across 100 seeds, got {}: {buckets:?}", buckets.len());
    }

    #[test]
    fn pick_transition_idx_stays_in_bounds() {
        for count in 1..=10 {
            for seed in 0u64..50 {
                let idx = pick_transition_idx(seed, count);
                assert!(idx < count, "idx {idx} out of range for count {count} seed {seed}");
            }
        }
    }
}
