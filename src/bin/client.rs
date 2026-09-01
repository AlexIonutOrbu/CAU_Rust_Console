use std::{
    error::Error,
    io::{self, BufReader, BufWriter, Write},
    net::TcpStream,
};

//client side
use cards_against_humanity_console::{Message, State};

fn main() -> Result<(), Box<dyn Error>> {
    // lettura info per connettersi al server
    let stdin = io::stdin();
    let mut ip_port = String::new();
    println!("Inserisci IP:Porta del server a cui vuoi connetterti (es. 127.0.0.1:8080)");
    io::stdout().flush()?;
    stdin.read_line(&mut ip_port)?;
    // connessione al server
    let stream: TcpStream = TcpStream::connect(ip_port.trim())?;
    println!("Connessione al server avvenuta con successo");
    /*
        Questo thread principale sarà sempre in ascolto dei messaggi che il client vorrà inviare al server
        Tuttavia per notificare il client di cosa dice il server necessitiamo di spawnare un thread che stia
        constantemente in ascolto sullo stream di lettura
    */
    let stream_lettura = stream.try_clone()?; // duplichiamo lo stream, in quanto dobbiamo dare quello di lettura ad un nuovo thread
    std::thread::spawn(move || {
        let reader = BufReader::new(stream_lettura); // buffer lettura
        // dal buffer lettura generiamo un iteratore che interpreta "Messagge"
        let mut iteratore = serde_json::Deserializer::from_reader(reader).into_iter::<Message>();
        while let Some(risultato) = iteratore.next() {
            match risultato {
                Ok(Message::Info { info }) => {
                    println!("{info}"); // ogni volta che riceve una info dal server si mettete in attesa
                }
                Ok(altro) => {
                    println!("\nMessaggio ricevuto non gestito a schermo: {:?}", altro);
                }
                Err(e) => {
                    eprintln!("\nConnessione con il server interrotta: {}", e);
                    break; // Esce dal ciclo se la connessione cade
                }
            }
        }
    });
    let mut writer = BufWriter::new(stream);
    let mut nickname = String::new();

    // Inserimento nickname
    println!("Inserisci il tuo nickname: ");
    io::stdout().flush()?;
    stdin.read_line(&mut nickname)?;
    nickname = nickname.trim().to_string();
    // invio messaggio di Connect
    let msg_connect = Message::Connect { nickname };
    serde_json::to_writer(&mut writer, &msg_connect)?;
    writer.flush()?;
    let mut messaggio = String::new();
    loop {
        io::stdout().flush()?;
        stdin.read_line(&mut messaggio)?;
        messaggio = messaggio.trim().to_string();
        if messaggio == "exit" {
            break;
        }
    }
    Ok(())
}