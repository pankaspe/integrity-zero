// src/game/state.rs

use super::node::{Node, NodeType};
use rand::seq::SliceRandom;
use rand::Rng; // Importiamo il "trait" Rng per avere accesso a .random_range()

#[derive(Debug)]
pub struct GameState {
    pub nodes: Vec<Node>,
    pub turn_count: u32,
    pub player_ap: u8,
    pub global_integrity: u8,
}

impl GameState {
    /// Crea una nuova istanza di GameState con una mappa generata proceduralmente.
    pub fn new() -> Self {
        let all_node_types = vec![
            NodeType::Gateway,
            NodeType::WebServer,
            NodeType::Authentication,
            NodeType::Database,
            NodeType::Firewall,
            NodeType::InternalApi,
        ];

        const GRID_SIZE: u8 = 4;
        const MIN_NODES: usize = 5;
        const MAX_NODES: usize = 6; // *** CORREZIONE IMPORTANTE ***

        // Il modo corretto per ottenere un generatore di numeri casuali.
        let mut rng = rand::rng();
        
        // Usiamo std::cmp::min per sicurezza, anche se ora MAX_NODES è 6.
        let num_nodes = rng.random_range(MIN_NODES..=std::cmp::min(MAX_NODES, all_node_types.len()));

        let mut nodes = Vec::with_capacity(num_nodes);
        let mut positions = Vec::new();

        let mut node_types_to_spawn = all_node_types; // Non serve più clonare
        node_types_to_spawn.shuffle(&mut rng);

        for i in 0..num_nodes {
            let mut pos = (rng.random_range(0..GRID_SIZE), rng.random_range(0..GRID_SIZE));
            while positions.contains(&pos) {
                pos = (rng.random_range(0..GRID_SIZE), rng.random_range(0..GRID_SIZE));
            }
            positions.push(pos);

            // Usiamo 'match' per gestire in modo sicuro il risultato di .pop()
            match node_types_to_spawn.pop() {
                Some(node_type) => {
                    // Questo blocco viene eseguito solo se .pop() restituisce un valore.
                    let name = format!("{:?}", node_type);
                    nodes.push(Node {
                        id: i,
                        node_type,
                        name,
                        hp: 100,
                        max_hp: 100,
                        pos,
                        status_effects: Vec::new(),
                    });
                }
                None => {
                    // Questo blocco viene eseguito se il vettore è vuoto.
                    // Non dovrebbe mai succedere, ma il nostro codice ora è sicuro!
                    // Possiamo lasciare vuoto o stampare un avviso per il debug.
                }
            }
        }

        Self {
            nodes,
            turn_count: 1,
            player_ap: 10,
            global_integrity: 50,
        }
    }
}