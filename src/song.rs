use midly::Smf;
use midly::{Event, EventKind, MidiMessage};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::midi_interpreter::{as_merged, to_abstime};

static DELTA_T: Duration = Duration::from_millis(1000 as u64);
static AFTERIMAGE: Duration = Duration::from_millis(1000 as u64);

#[derive(Debug, Copy, Clone)]
pub struct Tile {
    pub note: u8,
    pub start: Duration,
    pub length: Duration,
}

impl Tile {
    pub fn time_to_activate(&self, reference: &Instant) -> Duration {
        (*reference + self.start).saturating_duration_since(Instant::now())
    }

    pub fn time_to_die(&self, reference: &Instant) -> Duration {
        (*reference + self.start + self.length).saturating_duration_since(Instant::now())
    }

    pub fn vertical_position(&self, reference: &Instant, height: f32) -> f32 {
        height * self.time_to_die(reference).as_secs_f32() / DELTA_T.as_secs_f32()
            + self.vertical_height(height)
    }

    pub fn is_alive(&self, reference: &Instant) -> bool {
        (*reference + self.start - DELTA_T)
            .checked_duration_since(Instant::now())
            .is_none()
    }

    pub fn is_dead(&self, reference: &Instant) -> bool {
        (*reference + self.start + self.length)
            .checked_duration_since(Instant::now())
            .is_none()
    }

    pub fn in_scope(&self, reference: &Instant) -> bool {
        self.is_alive(reference) && !self.is_dead(reference)
    }

    pub fn vertical_height(&self, height: f32) -> f32 {
        self.length.as_secs_f32() / (DELTA_T.as_secs_f32() / height)
    }
}

/*#[derive(Debug, Copy, Clone)]
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
            start,
            length,
        }
    }

    /*pub fn from_iter<T: Iterator<Item = Tile>>(reference: &'a Instant, iter: T) -> impl Iterator {
        iter.map(|v| Self::new(reference, v.clone()))
    }*/

    pub fn time_to_activate(&self) -> Duration {
        (*self.reference + self.start).saturating_duration_since(Instant::now())
    }

    pub fn time_to_die(&self) -> Duration {
        (*self.reference + self.start + DELTA_T).saturating_duration_since(Instant::now())
    }

    pub fn vertical_position(&self, height: f32) -> f32 {
        self.time_to_die().as_secs_f32() / (DELTA_T.as_secs_f32() / height)
    }

    pub fn vertical_height(&self, height: f32) -> f32 {
        self.length.as_secs_f32() / (DELTA_T.as_secs_f32() / height)
    }
}*/

pub fn key_index(val: midly::number::u7) -> usize {
    let idx: u8 = val.into();
    idx as usize - 21
}

pub struct Song {
    pub target: PathBuf,
    pub tiles: Vec<Tile>,
}

impl Song {
    pub fn new<T: Into<PathBuf>>(tgt: T) -> Self {
        let target = tgt.into();
        let tiles = Self::process(target.clone());
        Self { target, tiles }
    }

    fn process(target: PathBuf) -> Vec<Tile> {
        let contents = fs::read(target.clone()).unwrap();
        let smf = Smf::parse(&contents).unwrap();
        let events = to_abstime(as_merged(smf.tracks));
        let mut starts: [Option<u32>; 128] = [None; 128];
        let mut tiles: Vec<Tile> = vec![];
        for ev in events {
            match ev.kind {
                EventKind::Midi {
                    channel: _,
                    message,
                } => match message {
                    MidiMessage::NoteOn { key, vel: _ } => {
                        let idx = key_index(key);
                        if starts[idx] == None {
                            starts[idx] = Some(ev.delta.into());
                        }
                    }
                    MidiMessage::NoteOff { key, vel: _ } => {
                        let idx = key_index(key);
                        let end: u32 = ev.delta.into();
                        if let Some(start) = starts[idx] {
                            let length = Duration::from_millis((end - start).into());
                            let tile = Tile {
                                note: idx as u8,
                                start: Duration::from_millis(start.into()),
                                length,
                            };
                            tiles.push(tile);
                            starts[idx] = None;
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
