use notify_rust::{
    Notification,
    Timeout,
};

use twitchchat::{
    TWITCH_IRC_ADDRESS,
    Decoder,
    Encodable,
    commands,
    messages::Commands,
    FromIrcMessage,
};

use oauth2::{
    AuthUrl,
    ClientId,
    CsrfToken,
    RedirectUrl,
    Scope,
};
use oauth2::basic::BasicClient;

use std::net::TcpStream;
use std::io::{
    self,
    prelude::*,
};

fn main() {
    let auth_token = get_oauth2_token();
    print!("Bot nickname (your twitch name): ");
    io::stdout().flush().expect("Failed to flush stdout...");
    let nick_name = io::stdin().lock().lines().next().unwrap().unwrap().to_lowercase();

    println!("connecting to {TWITCH_IRC_ADDRESS}");
    //let mut twitch_stream = TcpStream::connect(TWITCH_IRC_ADDRESS).unwrap();
    let mut twitch_stream = TcpStream::connect("irc.chat.twitch.tv:6667")
        .expect("Couldn't connect to server...");
    twitch_stream.set_read_timeout(None).expect("set_read_timeout call failed...");

    // Authentication / necessary steps
    println!("Logging in:");

    // PASS
    let pass_string = format!("PASS oauth:{auth_token}\r\n");
    twitch_stream.write_all(pass_string.as_bytes()).expect("Failed to write to stream...");
    print!("Client: {pass_string}");

    // theoretically I should read and verify the response here (if it exists)(?)

    // NICK
    let nick_string = format!("NICK {nick_name}\r\n");
    twitch_stream.write_all(nick_string.as_bytes()).expect("Failed to write to stream...");
    print!("Client: {nick_string}");

    // buf for reads
    let mut buf = [0; 511];

    // get response to authentication attempt (we're assuming it's good rn)
    let _bytes_read = twitch_stream.read(&mut buf).expect("Failed to read from stream...");
    println!("Server: {}", String::from_utf8_lossy(&buf));

    // verify here
    
    // join channel
    print!("Channel to join: ");
    io::stdout().flush().expect("Failed to flush stdout...");
    let channel = io::stdin().lock().lines().next().unwrap().unwrap();
    commands::join(&channel)
        .encode(&mut twitch_stream)
        .expect("Failed to write to stream...");
    print!("Client: JOIN #{channel}\r\n");
    
    // start the loop of reading from the server, respond to ping or other commands
    let mut decoder = Decoder::new(twitch_stream);
    loop {
        // get server response
        // if it turns out we're losing data, it's probably here
        let message = decoder.next()
            .expect("Failed to read message from stream...")
            .expect("Failed to read message from stream...");

        // print/log what we got
        print!("Server: {}", message.get_raw());

        // manage special commands
        match Commands::from_irc(message).expect("Failed to convert to command...") {
            Commands::Ping(cmd) => {
                // create resonse command:
                // "PONG :PING_data"(?)
                let command = commands::pong(cmd.token());
                // get the stream for writing and reinstate it to the decoder
                let mut stream = decoder.into_inner();
                command.encode(&mut stream).expect("Failed to write to stream...");
                decoder = Decoder::new(stream);
            },
            Commands::Privmsg(cmd) => {
                // lots of things i can suppoort here, PFPs potentially
                Notification::new()
                    .summary(cmd.display_name().unwrap_or(cmd.name()))
                    .body(cmd.data())
                    .timeout(Timeout::Default) // potentially change this depending on chat
                                                 // activity?
                    .show().expect("Failed to show notification...");
            },
            _ => {
            },
        }
//        let _bytes_read = twitch_stream.read(&mut buf).expect("Failed to read from stream...");
//        println!("Server: {}", String::from_utf8_lossy(&buf));
//        buf = [0; 511];
//
//        let _bytes_read = twitch_stream.read(&mut buf).expect("Failed to read from stream...");
//        println!("Server: {}", String::from_utf8_lossy(&buf));
//        buf = [0; 511];
//        let response = String::from_utf8_lossy(&buf);

//        for message in response.split_terminator("\r\n") {
//            println!("Server: {message}");
//        }
    }
//    let mut decoder = Decoder::new(twitch_stream);
//    
//    let msg = decoder.read_message();

//    loop {
//        Notification::new()
//            .summary("test notif")
//            .body("this notification is a test to see if im not stupid")
//            .timeout(Timeout::Milliseconds(1000*6)) // 6 seconds
//            .show().unwrap();
//
//        thread::sleep(Duration::new(1, 0));
//    };
}

fn get_oauth2_token() -> String {
    const CLIENT_ID: &str = "zt74bla70zzlpy6uj678tdm9jq0x59";
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None,
        AuthUrl::new("https://id.twitch.tv/oauth2/authorize".to_string()).unwrap(),
        None
    ).set_redirect_uri(RedirectUrl::new("https://localhost:3000".to_string()).unwrap());

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .use_implicit_flow()
        .add_scope(Scope::new("chat:read".to_string()))
        .url();

    println!("Browse to: {}", auth_url);
    print!("auth_token: ");
    io::stdout().flush().expect("Failed to flush stdout...");
    
    return io::stdin().lock().lines().next().unwrap().unwrap();
}
