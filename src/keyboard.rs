use midir::MidiInput;
use midir::MidiInputConnection;

pub struct Keyboard {
    conn: MidiInputConnection<()>,
}

impl Keyboard {
    pub fn new() -> Self {
        let midi_in = MidiInput::new("synthy reader").unwrap();
        let ports = midi_in.ports();
        let port = ports.iter().last().unwrap();
        let conn = midi_in
            .connect(
                port,
                "synthy-read-input",
                move |stamp, message, _| {
                    println!("{}: {:?} (len = {})", stamp, message, message.len());
                },
                (),
            )
            .unwrap();
        Self { conn }
    }
}
