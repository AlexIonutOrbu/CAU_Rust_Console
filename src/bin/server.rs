//server side

use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::os::linux::raw::stat;
use std::{
    self,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use cards_against_humanity_console::{Message, State};

#[derive(Clone)]
struct ServerInfo {
    lista_giocatori: Arc<Mutex<Vec<String>>>,
    giocatori_online: Arc<Mutex<Vec<String>>>,
    punteggi: Arc<Mutex<HashMap<String, u32>>>,
    stato: Arc<Mutex<State>>,
    mani_giocatori: Arc<Mutex<HashMap<String, Vec<String>>>>,
}
fn main() -> Result<(), Box<dyn Error>> {
    let server: TcpListener = TcpListener::bind("127.0.0.1:8080")?; // server in ascolto in localhost su porta locale
    let mut is_host = true;
    let info_comuni = ServerInfo {
        lista_giocatori: Arc::new(Mutex::new(Vec::new())),
        giocatori_online: Arc::new(Mutex::new(Vec::new())),
        punteggi: Arc::new(Mutex::new(HashMap::new())),
        stato: Arc::new(Mutex::new(State::Lobby)),
        mani_giocatori: Arc::new(Mutex::new(HashMap::new())),
    };
    for stream in server.incoming() {
        let stream = stream?; // Estrae lo stream in modo sicuro

        // Spawna un thread parallelo in modo che un giocatore lento non blocchi gli altri [2]
        let info_condivisa = info_comuni.clone();
        std::thread::spawn(move || {
            gestisci_client(stream, is_host, info_condivisa); // Passa la proprietà dello stream alla funzione e is_host, che implementa il tratto copy, sarà copiato
        });
        if is_host {
            is_host = false;
        }
    }
    Ok(())
}
fn gestisci_client(stream: TcpStream, is_host: bool, info: ServerInfo) {
    // 1. Qui dentro crei il BufReader e il BufWriter usando lo stream
    let reader = std::io::BufReader::new(&stream);
    let mut writer = std::io::BufWriter::new(&stream);
    // 2. Inizializzi l'iteratore JSON
    let mut iteratore = serde_json::Deserializer::from_reader(reader).into_iter::<Message>();
    let mut nickname_salvato: Option<String> = None;
    // 3. Avvii il ciclo di ascolto dei messaggi (il loop che rimarrà attivo)
    while let Some(risultato) = iteratore.next() {
        match risultato {
            Ok(Message::Connect { nickname }) => {
                println!("Si è connesso il client {nickname}");
                let stato_guard = info.stato.lock().unwrap(); // accesso allo stato della partita
                match &*stato_guard {
                    State::Lobby => {
                        drop(stato_guard); //
                        let mut lista_guard = info.lista_giocatori.lock().unwrap();
                        let (nickname_effettivo, modificato) = if lista_guard.contains(&nickname) {
                            println!("Nickname già in uso: {nickname}");
                            (format!("{}{}", nickname, lista_guard.len()), true) // Restituisce il nome modificato
                        } else {
                            (nickname, false) // Restituisce il nome originale (spostando l'ownership)
                        };
                        lista_guard.push(nickname_effettivo.clone());
                        nickname_salvato=Some(nickname_effettivo.clone());
                        let mut punteggi_guard = info.punteggi.lock().unwrap();
                        punteggi_guard.insert(nickname_effettivo.clone(), 0);

                        println!(
                            "Giocatore {nickname_effettivo} aggiunto alla lista dei giocatori"
                        );
                        // Rilasciamo i lock sui punteggi e lista_giocatori prima di inviare il messaggio di conferma al client
                        drop(punteggi_guard);
                        drop(lista_guard);
                        let messaggio_conferma = if modificato {
                            Message::Info {
                                info: format!(
                                    "Nickname già in uso. Il tuo nuovo nickname è: {nickname_effettivo}"
                                ),
                            }
                        } else {
                            Message::Info {
                                info: format!("Benvenuto, {nickname_effettivo}!"),
                            }
                        };
                        if let Err(e) = serde_json::to_writer(&mut writer, &messaggio_conferma) {
                            // proviamo ad inviare il messaggio serializzato
                            eprintln!("Errore nella serializzazione del messaggio di Info: {e}");
                        } else if let Err(e) = writer.flush() {
                            // esegue il flush dopo aver inviato i dati
                            eprintln!("Errore nell'invio (flush) del messaggio di Info: {e}");
                        }
                    }
                    _=> {
                        drop(stato_guard);
                        let lista_guard = info.lista_giocatori.lock().unwrap();
                        let mut lista_online_guard = info.giocatori_online.lock().unwrap();

                        if lista_guard.contains(&nickname) && !lista_online_guard.contains(&nickname) {
                            //Riconnessione valida
                            lista_online_guard.push(nickname.clone());
                            
                            nickname_salvato = Some(nickname.clone());

                            drop(lista_online_guard);
                            drop(lista_guard);

                            let messaggio_conferma = Message::Info {
                                info: format!("Bentornato in partita, {nickname}!"),
                            };
                            let _ = serde_json::to_writer(&mut writer, &messaggio_conferma);
                            let _ = writer.flush();
                        } else {
                            // Accesso negato
                            drop(lista_online_guard);
                            drop(lista_guard);

                            let messaggio_conferma = Message::Info {
                                info: "Non è possibile connettersi ad una partita già iniziata o riconnettersi usando il nickname di un altro giocatore".to_string(),
                            };
                            let _ = serde_json::to_writer(&mut writer, &messaggio_conferma);
                            let _ = writer.flush();
                        }
                        
                    }
                }
            }
            Ok(Message::Start)=>{
                println!("Ricevuto un messaggio start");
                let stato_guard = info.stato.lock().unwrap(); // accesso allo stato della partita
                match &*stato_guard {
                    State::Lobby =>{
                        drop(stato_guard);
                        if is_host {
                            let mut stato =info.stato.lock().unwrap();
                            *stato=State::Creating;
                            let messaggio_conferma = Message::Info {
                                info: "Creazione partita in corso".to_string(),
                            };
                            let _ = serde_json::to_writer(&mut writer, &messaggio_conferma);
                            let _ = writer.flush();
                            // TODO creazione partita
                        }else{
                            let messaggio_conferma = Message::Info {
                                info: "Non sei l'host".to_string(),
                            };
                            let _ = serde_json::to_writer(&mut writer, &messaggio_conferma);
                            let _ = writer.flush();
                        }
                    }
                    _ =>{}
                }
            }
            Ok(altro_messaggio) => {}
            Err(errore) => {
                eprintln!("Errore di lettura o disconnessione brusca: {errore}");
            }
        }
    }

    /*
        Quando il ciclo termina (il client si disconnette), la funzione finisce. Tuttavia c'è la possibilità di una disconnessione non voluta
        Proprio per questo verrà aggiunto alla lista dei giocatori disconnessi
    */ 
    if let Some(ref name) = nickname_salvato {
        let stato_attuale = info.stato.lock().unwrap();

        match &*stato_attuale {
            State::Lobby=>{
                // se siamo in lobby il giocatore viene rimosso direttamente
                println!{"Giocatore {name} rimosso dalla lobby"};
                info.lista_giocatori.lock().unwrap().retain(|x| x != name);
                info.punteggi.lock().unwrap().remove(name);
                info.giocatori_online.lock().unwrap().retain(|x| x != name);
            }
            altro=>{
                println!{"Giocatore {name} disconnesso"};
                info.giocatori_online.lock().unwrap().retain(|x| x != name);
            }
        }
    }
}
