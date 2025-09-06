// src/game/node.rs

// L'attributo `derive` chiede al compilatore di implementare automaticamente
// alcuni "trait" (funzionalità) per noi.
// - Debug: Ci permette di stampare la struct in modo leggibile (con {:?}).
// - Clone: Ci permette di creare copie esatte dell'oggetto.
// - PartialEq: Ci permette di confrontare due istanze con `==`.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Gateway,
    WebServer,
    Authentication,
    Database,
    Firewall,
    InternalApi,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusEffect {
    Shield,       // Armatura da Fortify
    Vulnerability, // Danni subiti +50%
    Malware,      // Riduce gli AP del giocatore
    Backdoor,     // Bomba a tempo
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize, // Un identificatore numerico unico
    pub node_type: NodeType,
    pub name: String,
    pub hp: u8,
    pub max_hp: u8,
    pub pos: (u8, u8), // Posizione (x, y) sulla griglia 4x4
    pub status_effects: Vec<StatusEffect>,
}