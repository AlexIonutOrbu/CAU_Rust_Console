//server side

use std::{self, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}};
use std::error::Error;
use std::collections::HashMap;

use cards_against_humanity_console::{Message,State};

#[derive(Clone)]
struct serverInfo{
    lista_giocatori: Arc<Mutex<Vec<String>>>,
    punteggi: Arc<Mutex<HashMap<String, u32>>>,
    stato:Arc<Mutex<State>>,
    mani_giocatori: Arc<Mutex<HashMap<String, Vec<String>>>>,
}
fn main() -> Result<(), Box<dyn Error>> {
    let server:TcpListener=TcpListener::bind("127.0.0.1:8080")?; // server in ascolto in localhost su porta locale
    let mut is_host=true;
    let info_comuni=serverInfo{
        lista_giocatori:Arc::new(Mutex::new(Vec::new())), 
        punteggi: Arc::new(Mutex::new(HashMap::new())),
        stato: Arc::new(Mutex::new(State::Lobby)),
        mani_giocatori: Arc::new(Mutex::new(HashMap::new())),
    };
    for stream in server.incoming() {
        let stream = stream?; // Estrae lo stream in modo sicuro
    
        // Spawna un thread parallelo in modo che un giocatore lento non blocchi gli altri [2]
        let info_condivisa=info_comuni.clone();
        std::thread::spawn(move || {
            gestisci_client(stream, is_host, info_condivisa); // Passa la proprietà dello stream alla funzione e is_host, che implementa il tratto copy, sarà copiato
        });
        if is_host {
            is_host=false;
        }
    }
    Ok(())
}
fn gestisci_client(stream: TcpStream, is_host:bool, info:serverInfo) {
    // 1. Qui dentro crei il BufReader e il BufWriter usando lo stream
    let reader = std::io::BufReader::new(&stream);
    let mut writer =std::io::BufWriter::new(&stream);
    // 2. Inizializzi l'iteratore JSON
    let mut iteratore = serde_json::Deserializer::from_reader(reader).into_iter::<Message>();
    
    // 3. Avvii il ciclo di ascolto dei messaggi (il loop che rimarrà attivo)
    while let Some(risultato) = iteratore.next() {
        match risultato {
            Ok(Message::Connect { nickname })=>{

            },
            Ok(altro_messaggio)=>{

            }
            Err(errore)=>{

            }
        }
    }
    
    // Quando il ciclo termina (il client si disconnette), la funzione finisce.
    // Lo `stream` uscirà dallo scope e verrà rimosso dalla memoria (dropped),
    // chiudendo automaticamente la connessione TCP [1].
}