use midly::Smf;
use midly::{Event, EventKind, MidiMessage};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::midi_interpreter::{as_merged, to_abstime};

#[derive(Debug, Copy, Clone)]
struct Tile {
    note: u8,
    start: u32,
    length: Duration,
}

#[derive(Debug, Copy, Clone)]
struct TickingTile<'a> {
    reference: &'a Instant,
    note: u8,
    start: Duration,
    length: Duration,
}

impl<'a> TickingTile<'a> {
    pub fn new(reference: &'a Instant, tile: Tile) -> Self {
        let Tile {
            note,
            start,
            length,
        } = tile;
        Self {
            reference,
            note,
            start: Duration::from_millis(start.into()),
            length,
        }
    }

    pub fn time_to_activate(&self) -> Duration {
        (*self.reference + self.start).saturating_duration_since(Instant::now())
    }
}

pub struct Song {
    target: PathBuf,
}

impl Song {
    pub fn new<T: Into<PathBuf>>(tgt: T) -> Self {
        let target = tgt.into();
        let mut tiles = Self::process(target.clone());
        tiles.sort_by_key(|i| i.start);
        tiles.iter().for_each(|i| println!("{:?}", i));
        /*std::thread::spawn(move || {
            let reference = Instant::now();
            let ticker = tiles
                .iter()
                .map(|v| TickingTile::new(&reference, *v))
                .last()
                .unwrap();
            loop {
                println!("{:?}", ticker.time_to_activate());
            }
        });*/
        Self { target }
    }

    fn process(tgt: PathBuf) -> Vec<Tile> {
        let contents = fs::read(tgt).unwrap();
        let smf = Smf::parse(&contents).unwrap();
        let events = to_abstime(as_merged(smf.tracks));
        let mut starts: [Option<u32>; 128] = [None; 128];
        let mut tiles: Vec<Tile> = vec![];
        for ev in events {
            match ev.kind {
                EventKind::Midi { channel, message } => match message {
                    MidiMessage::NoteOn { key, vel: _ } => {
                        let idx: u8 = key.into();
                        if starts[idx as usize] == None {
                            starts[idx as usize] = Some(ev.delta.into());
                        }
                    }
                    MidiMessage::NoteOff { key, vel: _ } => {
                        let idx: u8 = key.into();
                        let end: u32 = ev.delta.into();
                        if let Some(start) = starts[idx as usize] {
                            let length = Duration::from_millis((end - start).into());
                            let tile = Tile {
                                note: key.into(),
                                start,
                                length,
                            };
                            tiles.push(tile);
                            starts[idx as usize] = None;
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
        tiles
    }
}
