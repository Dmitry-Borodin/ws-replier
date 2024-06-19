use tungstenite::{connect, Message};

fn main() {
    env_logger::init();

    let (mut socket, response) = connect("ws://34.159.18.133:22430/").expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");

    loop {
        let message = socket.read().expect("Error reading message");
        // println!("Received: {:?}", message.into_data());

        match message {
            msg @ Message::Text(_) => {
                socket.send(msg)?;
            }
            msg @ Message::Binary(_) => {
                socket.send(msg)?;
            }
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) | Message::Frame(_) => {}
        }
    }
}

fn get_reply(challenge: Vec<u8>) -> Result<Vec<u8>> {

}