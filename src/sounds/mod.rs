use std::io::Cursor;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Source, StreamError, decoder::DecoderError, source::Buffered};

pub struct Sounds {
    stream: OutputStream,
}

impl Sounds {
    pub fn new() -> Result<Self, StreamError> {
        Ok(Self {
            stream: OutputStreamBuilder::open_default_stream()?
        })
    }

    pub fn play<S>(&self, source: S) 
    where 
        S: Source + Send + 'static,
        S::Item: rodio::cpal::Sample,
        f32: rodio::cpal::FromSample<S::Item>, 
    {
        self.stream.mixer().add(source);
    }
}

pub struct Sound {
    buffered_sound: Buffered<Decoder<Cursor<&'static [u8]>>>,
}

impl Sound {
    pub fn new(bytes: &'static [u8]) -> Result<Self, DecoderError> {
        Ok(Self {
            buffered_sound: Decoder::new_wav(Cursor::new(bytes))?.buffered()
        })
    }

    pub fn get(&self) -> Buffered<Decoder<Cursor<&'static [u8]>>> {
        self.buffered_sound.clone()
    }
}
