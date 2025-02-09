use clap::{Parser, Subcommand};
use r3bl_ansi_color::SgrCode;
use std::thread;
use std::{
    io::{stdin, BufRead, BufReader, BufWriter, Write},
    net::{IpAddr, TcpListener, TcpStream},
};


use type_aliases::*;
mod type_aliases {
    pub type IOResult<T> = std::io::Result<T>;
}

use defaults::*;
mod defaults{
    pub const DEFAULT_PORT: u16 = 3000;
    pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
}

pub use clap_config::*;
mod clap_config {
    use super::*;

    #[derive(Parser, Debug)]
    pub struct CLIArg {
        #[clap(long,short, default_value = DEFAULT_ADDRESS, global = true)]
        pub address : IpAddr,

        #[clap(long, short, default_value_t = DEFAULT_PORT , global = true )]
        pub port : u16,

        #[clap(long,short = 'd', global = true)]
        pub log_disable : bool,

        #[clap(subcommand)]
        pub subcommand : CLISubcommand,
    }

    #[derive(Subcommand, Debug)]
    pub enum CLISubcommand {
        Server,
        Client,
    }
}

fn main() {
    println!("Welcome to rtelnet");

    let cli_arg = CLIArg::parse();
    let address = cli_arg.address;
    let port = cli_arg.port;
    let socket_addr = format!("{},{}", address, port);

    if !cli_arg.log_disable {
        femme::start();
    }

    match  match cli_arg.subcommand {
        CLISubcommand::Server => start_server(socket_addr),
        CLISubcommand::Client => start_client(socket_addr),
    } {
        Ok(_) => { 
            println!("Program Exited Successfully");
        }
        Err(error) => {
            println!("Progam exited with an error {}",error);

        }
    }
}



use server::*;
mod server {

    use super::*;

    pub fn start_server(socket_addr : String) -> IOResult<()> {
        let tcp_listener = TcpListener::bind(socket_addr)?;

        loop {
            log::info!("Waiting for a incoming connection...");
            let (tcp_listener, ..) = tcp_listener.accept()?;


            thread::spawn(|| match handle_connection(tcp_listener){
                Ok(_) => {
                    log::info!("Successfully closed connection to client");
                }
                Err(_) => {
                    log::error!("Problem with client connection....");
                }
            });
        }


    }

    fn handle_connection(tcp_stream : TcpStream) -> IOResult<()> {
        log::info!("Start handle connections");

        let reader = &mut BufReader::new(&tcp_stream);
        let write  = &mut BufWriter::new(&tcp_stream);

        loop {
            let mut incoming : Vec<u8> = vec![];

            let num_bytes_read = reader.read_until(b'\n', &mut incoming)?;

            if num_bytes_read == 0 {
                break;
            }

            let outgoing = process(&incoming);

            write.write(&outgoing)?;

            let _ = write.flush()?;

            log::info!("-> rx(bytes) : {:?}", &incoming);

            log::info!(
                "-> rx(string) : '{}', size: {} bytes ",
                String::from_utf8_lossy(&incoming).trim(),
                incoming.len(),
            );

            log::info!(
                "<- tx(string) : {} , size : {} bytes",
                String::from_utf8_lossy(&outgoing).trim(),
                outgoing.len(),
            );
        }
        log::info!("End handle connection - connection closed");

        Ok(())
    }

    fn process(incoming : &Vec<u8>) -> Vec<u8> {
        let incoming = String::from_utf8_lossy(incoming);

        let incoming = incoming.trim();

        let outgoing = incoming.to_string();

        let outgoing = format!("{}\n", outgoing);

        outgoing.as_bytes().to_vec()
    }

}



fn start_client(socket_addr : String) -> IOResult<()> {
    log::info!("Start client connections");
    let tcp_stream = TcpStream::connect(socket_addr)?;
    let (mut reader,  mut writer) = (BufReader::new(&tcp_stream),BufWriter::new(&tcp_stream));

    loop {
        let outgoing = {
            let mut it = String::new();
            let _ = stdin().read_line(&mut it)?;
            it.into_bytes().to_vec()
        };

        let _ = writer.write(&outgoing)?;
        writer.flush()?;

        let incoming = {
            let mut it = vec![];
            let _ = reader.read_until(b'\n', &mut it);
            it
        };
        

        let display_msg = String::from_utf8_lossy(&incoming);

        let display_msg = display_msg.trim();

        let reset = SgrCode::Reset.to_string();
        let display_msg = format!("{} {}", display_msg,reset);
        println!("{}",  display_msg);

        log::info!(
            "-> tx : {}, size : {} bytes {}", 
            String::from_utf8_lossy(&outgoing).trim(),
            outgoing.len(),
            reset
        );
        
        log::info!(
            "<- rx : {} , size : {} bytes {}",
            String::from_utf8_lossy(&incoming).trim(),
            incoming.len(),
            reset,
        );
    }
}