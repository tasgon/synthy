use midly::Smf;
use midly::{Event, EventKind, MidiMessage};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::midi_interpreter::{as_merged, to_abstime};

//static mut MILLIS: u32 = 2000;
static mut DELTA_T: Duration = Duration::from_millis(2000 as u64);
//static AFTERIMAGE: Duration = Duration::from_millis(1000 as u64);

pub fn deltat() -> Duration {
    unsafe { DELTA_T }
}

pub fn set_deltat(ms: impl Into<u64>) {
    unsafe {
        DELTA_T = Duration::from_millis(ms.into());
    }
}

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
        height * (1.0 - self.time_to_die(reference).as_secs_f32() / deltat().as_secs_f32())
    }

    pub fn is_alive(&self, reference: &Instant) -> bool {
        (*reference + self.start - deltat())
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
        height * self.length.as_secs_f32() / deltat().as_secs_f32()
    }
}

pub fn key_index<T: Into<u8>>(val: T) -> usize {
    let idx: u8 = val.into();
    idx as usize - 21
}

pub struct Song {
    pub target: PathBuf,
    pub tiles: Vec<Tile>,
    pending_tiles: Vec<Tile>,
    pub active_tiles: Vec<Tile>,
    //pub reference: Instant,
}

impl Song {
    pub fn new<T: Into<PathBuf>>(tgt: T) -> Self {
        let target = tgt.into();
        let mut tiles = Self::process(target.clone());
        tiles.sort_by_key(|i| i.start);
        let mut pending_tiles = tiles.clone();
        let len = tiles.len();
        pending_tiles.reverse();
        Self {
            target,
            tiles,
            pending_tiles,
            active_tiles: Vec::with_capacity(len),
            //reference: Instant::now(),
        }
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

    pub fn update(&mut self, reference: &Instant) {
        //let mut v: Vec<Tile> = Vec::with_capacity(self.tiles.len());
        while self
            .pending_tiles
            .last()
            .map(|i| i.is_alive(reference))
            .unwrap_or(false)
        {
            self.active_tiles.push(self.pending_tiles.pop().unwrap());
        }

        while self
            .active_tiles
            .get(0)
            .map(|i| i.is_dead(reference))
            .unwrap_or(false)
        {
            self.active_tiles.remove(0);
        }
    }
}
