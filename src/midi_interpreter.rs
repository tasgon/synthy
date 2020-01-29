use midly::{Event, EventKind, MetaMessage};
use std::vec::Vec;

// Algorithms and concepts taken from  'https://github.com/mido/mido/', covered by the MIT license

/// Convert all events' delay items from relative to absolute timings.
pub fn to_abstime(v: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut now: u32 = 0;
    v.iter()
        .map(move |i: &Event<'_>| {
            let delta: u32 = i.delta.into();
            now += delta;
            let mut n = i.clone();
            n.delta = now.into();
            n
        })
        .collect()
}

/// Convert all events' delay items from absolute to relative timings.
pub fn to_reltime(v: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut now: u32 = 0;
    v.iter()
        .map(move |i: &Event<'_>| {
            let delta: u32 = i.delta.into();
            let mut n = i.clone();
            n.delta = (delta - now).into();
            now = delta;
            n
        })
        .collect()
}

/// Remove all extra `EndOfTrack` messages.
pub fn fix_track_end(v: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut out: Vec<Event<'_>> = vec![];
    let mut accum: u32 = 0;
    for msg in v {
        if msg.kind == EventKind::Meta(MetaMessage::EndOfTrack) {
            let delta: u32 = msg.delta.into();
            accum += delta;
        } else {
            let mut ev = msg.clone();
            if accum > 0 {
                let delta: u32 = msg.delta.into();
                ev.delta = (delta + accum).into();
            }
            out.push(ev)
        }
    }
    out.push(Event {
        delta: accum.into(),
        kind: EventKind::Meta(MetaMessage::EndOfTrack),
    });
    out
}

/// Convert an array of tracks to one array containing all the events.
pub fn as_merged(v: Vec<Vec<Event<'_>>>) -> Vec<Event<'_>> {
    let mut messages: Vec<Event<'_>> = vec![];
    v.iter()
        .for_each(|i| messages.extend(to_abstime(i.clone())));
    messages.sort_by_key(|i: &Event<'_>| {
        let n: u32 = i.delta.into();
        n
    });
    fix_track_end(to_reltime(messages))
}
