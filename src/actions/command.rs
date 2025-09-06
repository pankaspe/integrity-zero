// src/actions/command.rs

// Definiamo un enum per tutti i comandi possibili.
// Questo ci permette di gestire i comandi in modo sicuro e strutturato.
#[derive(Debug, PartialEq)]
pub enum Command {
    Help(Option<String>), // `help` può avere un argomento opzionale
    Fortify(String),
    Scan(String),
    // ... altri comandi in futuro ...
    Quit,
    Invalid, // Per comandi non riconosciuti
}

// La nostra funzione di parsing.
// Prende una stringa e restituisce un `Command`.
pub fn parse_command(input: &str) -> Command {
    // Semplice parsing per ora: cerchiamo `(argomento)`.
    if let Some(open_paren) = input.find('(') {
        if let Some(close_paren) = input.find(')') {
            let command_name = &input[..open_paren];
            let arg = &input[open_paren + 1..close_paren];

            return match command_name.trim() {
                "help" => Command::Help(if arg.is_empty() { None } else { Some(arg.to_string()) }),
                "fortify" => Command::Fortify(arg.to_string()),
                "scan" => Command::Scan(arg.to_string()),
                _ => Command::Invalid,
            };
        }
    }

    // Gestisce comandi senza parentesi, come 'q' o 'quit'
    match input.trim() {
        "q" | "quit" => Command::Quit,
        "help" => Command::Help(None), // `help` senza parentesi
        _ => Command::Invalid,
    }
}