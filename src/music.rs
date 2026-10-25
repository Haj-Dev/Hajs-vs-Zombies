//! Streamed background music.
//!
//! The DSP can only play raw PCM, and the tracks are far too large to fit in
//! memory, so we stream them: the file is read one chunk at a time into buffers
//! allocated in LINEAR memory and queued on a dedicated NDSP channel.

use std::{
    fs::File,
    io::{ErrorKind, Read, Seek, SeekFrom},
};

use ctru::{
    linear::LinearAllocator,
    services::ndsp::{
        AudioFormat, AudioMix, Channel, InterpolationType, Ndsp,
        wave::{Status, Wave},
    },
};

use crate::scene::Track;

/// Sample rate of every track in `romfs:/musics`.
const SAMPLE_RATE: f32 = 22050.;

const FORMAT: AudioFormat = AudioFormat::PCM16Mono;

/// One second of mono PCM16.
const CHUNK: usize = SAMPLE_RATE as usize * FORMAT.size();

/// Number of chunks queued ahead of the DSP, i.e. the amount of slack a slow
/// frame can eat before the music stutters.
const BUFFERS: usize = 3;

type Buffer = Vec<u8, LinearAllocator>;

pub struct Music<'n> {
    channel: Channel<'n>,
    /// `None` while the slot's previous wave is still queued or playing.
    waves: [Option<Wave<Buffer>>; BUFFERS],
    file: Option<File>,
    current: Option<Track>,
}

/// Apply the channel setup. `Channel::reset` wipes it, so this runs again
/// after every reset.
fn configure(channel: &mut Channel<'_>) {
    channel.set_interpolation(InterpolationType::Linear);
    channel.set_sample_rate(SAMPLE_RATE);
    channel.set_format(FORMAT);

    let mut mix = AudioMix::zeroed();
    mix.set_front(1.0, 1.0);
    channel.set_mix(&mix);
}

impl<'n> Music<'n> {
    /// Take ownership of a channel, configured for [`FORMAT`] at [`SAMPLE_RATE`].
    pub fn new(ndsp: &'n Ndsp) -> Result<Self, ctru::services::ndsp::Error> {
        let mut channel = ndsp.channel(0)?;
        configure(&mut channel);

        Ok(Self {
            channel,
            waves: std::array::from_fn(|_| None),
            file: None,
            current: None,
        })
    }

    /// Play `wanted`, keeping the current track if it is already the one asked
    /// for. Call once per frame.
    pub fn play(&mut self, wanted: Option<Track>) {
        if wanted != self.current {
            // Stop the DSP immediately, then drop the waves it was reading
            // (which also releases their LINEAR buffers).
            self.channel.reset();
            configure(&mut self.channel);
            self.waves = std::array::from_fn(|_| None);
            self.file = wanted.and_then(|track| File::open(track.path()).ok());
            self.current = wanted;
        }

        self.refill();
    }

    /// Replace every finished buffer with the next chunk of the track.
    fn refill(&mut self) {
        let Some(file) = self.file.as_mut() else {
            return;
        };

        for slot in self.waves.iter_mut() {
            let reusable = match slot {
                None => true,
                Some(wave) => matches!(wave.status(), Status::Free | Status::Done),
            };
            if !reusable {
                continue;
            }

            *slot = None;
            let Some(buffer) = next_chunk(file) else {
                return;
            };

            // Put the wave in its final home *before* queueing it: the DSP
            // keeps a pointer to it, so it must not move afterwards.
            let wave = slot.insert(Wave::new(buffer, FORMAT, false));
            if self.channel.queue_wave(wave).is_err() {
                *slot = None;
                return;
            }
        }
    }
}

/// Read a full chunk, wrapping around to the start of the file at EOF so the
/// track loops seamlessly.
fn next_chunk(file: &mut File) -> Option<Buffer> {
    let mut buffer: Buffer = Vec::with_capacity_in(CHUNK, LinearAllocator);
    buffer.resize(CHUNK, 0);

    let mut filled = 0;
    while filled < CHUNK {
        match file.read(&mut buffer[filled..]) {
            Ok(0) => {
                file.seek(SeekFrom::Start(0)).ok()?;
            }
            Ok(read) => filled += read,
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(_) => return None,
        }
    }

    Some(buffer)
}
