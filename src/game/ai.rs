// src/game/ai.rs

use super::node::Node;
// Usiamo il prelude di `rand` per importare tutti i trait necessari, inclusi Rng e SliceRandom.
use rand::prelude::*;

#[derive(Debug, Clone)]
pub enum AiAction {
    Exploit(usize),
    Weaken(usize),
}

pub struct AiMind;

impl AiMind {
    pub fn decide_action(nodes: &[Node]) -> Option<AiAction> {
        // Ora che il trait corretto è in scope, .choose() funzionerà.
        if let Some(target_node) = nodes.choose(&mut rand::rng()) {
            let available_actions = vec![
                AiAction::Exploit(target_node.id),
                AiAction::Weaken(target_node.id),
            ];
            return available_actions.choose(&mut rand::rng()).cloned();
        }
        None
    }
}