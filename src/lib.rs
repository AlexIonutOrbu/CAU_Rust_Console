//libraries

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum State {
    Lobby,           // stato in cui il server è in attesa che i giocatori si connettano
    Creating, // stato in cui il server sta creando il mazzo, distribuendo le mani ai giocatori e preparando il primo Statesync
    WaitingForCards, // stato in cui il server è in attesa delle scelte dei giocatori per completare la carta bianca
    Judging,         // stato in cui il server è in attesa della scelta del giudice
    StateChanging,   // messaggi per sincronizzare client e server fornendo aggiornamenti
    Finish,          // stato di chiusura
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Message {
    Connect {
        nickname: String,
    },
    Start,
    ChoiceList {
        choices: Vec<String>,
        number_of_choices: u32,
    },
    Choice {
        selected: u32,
    },
    StateSync {
        state: State,
        players_turn: Vec<String>,
        current_white_card: Option<String>,
        current_player_winner: Option<String>,
        current_option_winner: Option<String>,
        current_player_rankings: HashMap<String, u32>,
    },
    Finish,
}
