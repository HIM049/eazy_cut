use std::{
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use anyhow::anyhow;
use ffmpeg_next::{
    Rational,
    codec::{self},
    decoder::{self},
    format::{self, context},
    frame::audio,
    software::scaling::{self},
};
use gpui::{Context, Entity};
use ringbuf::{
    HeapProd,
    traits::{Observer, Producer},
};

use crate::{
    models::model::OutputParams,
    ui::{
        player::{frame::FrameImage, size::PlayerSize, utils::generate_image_fallback},
        views::app::MyApp,
    },
};

#[derive(Debug)]
pub enum DecoderEvent {
    None,
    Stop,
    Pause,
    Seek(f32),
}

pub struct VideoDecoder {
    input: Option<context::Input>,
    video_stream_ix: usize,
    audio_stream_ix: usize,
    v_decoder: Option<decoder::Video>,
    a_decoder: Option<decoder::Audio>,
    time_base: Rational,
    duration: i64,

    producer: Option<HeapProd<FrameImage>>,
    size: Entity<PlayerSize>,
    output_prarms: Entity<OutputParams>,

    event: Arc<Mutex<DecoderEvent>>,
    condvar: Arc<Condvar>,
}

impl VideoDecoder {
    // /// Create a new Decoder
    // pub fn new(size_entity: Entity<PlayerSize>, output_prarms: Entity<OutputParams>) -> Self {
    //     Self {
    //         input: None,
    //         video_stream_ix: 0,
    //         audio_stream_ix: 0,
    //         decoder: None,
    //         code_parms: codec::Parameters::new(),
    //         time_base: None,
    //         frames: 0,
    //         duration: 0,

    //         producer: None,
    //         size: size_entity,
    //         output_prarms,

    //         event: Arc::new(Mutex::new(DecoderEvent::None)),
    //         condvar: Arc::new(Condvar::new()),
    //     }
    // }

    /// set producer of ringbuf in VideoDecoder
    pub fn set_producer(mut self, p: HeapProd<FrameImage>) -> Self {
        self.producer = Some(p);
        self
    }

    /// set DecoderEvent
    pub fn set_event(&mut self, new: DecoderEvent) {
        let mut event = self.event.lock().unwrap();
        *event = new;
        self.condvar.notify_all();
    }

    /// get video timebase
    pub fn get_timebase(&self) -> Rational {
        self.time_base
    }

    pub fn get_duration(&self) -> Option<i64> {
        if self.duration == 0 {
            return None;
        }
        Some(self.duration)
    }

    /// open a video file
    pub fn open<T>(
        cx: &mut Context<T>,
        path: &PathBuf,
        size: Entity<PlayerSize>,
        output_prarms: Entity<OutputParams>,
    ) -> anyhow::Result<Self>
    where
        T: 'static,
    {
        let i = ffmpeg_next::format::input(path)?;

        let v_stream = i
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .ok_or(anyhow!("failed to find best video stream"))?;

        let a_stream = i
            .streams()
            .best(ffmpeg_next::media::Type::Audio)
            .ok_or(anyhow!("failed to find video stream"))?;

        let decoder = ffmpeg_next::codec::context::Context::from_parameters(v_stream.parameters())?
            .decoder()
            .video()?;

        let audio_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(a_stream.parameters())?
                .decoder()
                .audio()?;

        let time_base = v_stream.time_base();

        // get sample rate and length of video frams
        let frame_rate = v_stream.avg_frame_rate();
        let duration = v_stream.duration();
        // get orignal video size
        let orignal_width = decoder.width();
        let orignal_height = decoder.height();

        println!("DEBUG: frame rate: {}, duration: {}", frame_rate, duration);

        size.update(cx, |s, _| {
            s.set_orignal((orignal_width, orignal_height));
        });

        // update related output params
        output_prarms.update(cx, |p, _| {
            p.path = Some(path.clone());
            p.video_stream_ix = Some(v_stream.index());
            p.audio_stream_ix = Some(a_stream.index());
        });

        Ok(Self {
            video_stream_ix: v_stream.index(),
            audio_stream_ix: a_stream.index(),
            v_decoder: Some(decoder),
            a_decoder: Some(audio_decoder),
            time_base,
            duration,
            producer: None,
            size,
            output_prarms,
            input: Some(i),

            event: Arc::new(Mutex::new(DecoderEvent::None)),
            condvar: Arc::new(Condvar::new()),
        })
    }

    /// spawn decoder thread
    pub fn spawn_decoder(&mut self, size: Entity<PlayerSize>, cx: &mut Context<MyApp>) {
        let Some(mut input) = self.input.take() else {
            return;
        };
        let Some(mut decoder) = self.v_decoder.take() else {
            return;
        };
        let Some(mut producer) = self.producer.take() else {
            return;
        };

        let time_base = self.time_base;

        let orignal_size = size.read(cx).orignal_size();

        let video_ix = self.video_stream_ix;
        let audio_ix = self.audio_stream_ix;

        let w = decoder.width();
        let h = decoder.height();
        let event = self.event.clone();
        let condvar = self.condvar.clone();
        thread::spawn(move || {
            // init ffmpeg scaler
            let mut scaler_context = ffmpeg_next::software::scaling::Context::get(
                decoder.format(),
                w,
                h,
                format::Pixel::BGRA,
                w,
                h,
                scaling::Flags::BILINEAR,
            )
            .unwrap();

            // frame buffer
            let mut frame_buf: Option<FrameImage> = None;
            // frame varible
            let mut decoded_frame = ffmpeg_next::frame::Video::new(decoder.format(), w, h);
            let mut scaled_frame = ffmpeg_next::frame::Video::new(format::Pixel::BGRA, w, h);

            let mut seek_to: Option<f32> = None;
            loop {
                {
                    // handle decoder event
                    let mut event = event.lock().unwrap();
                    match *event {
                        DecoderEvent::None => (),
                        DecoderEvent::Stop => break,
                        DecoderEvent::Pause => {
                            let _event = condvar.wait(event).unwrap();
                            continue;
                        }
                        DecoderEvent::Seek(t) => {
                            let ts = (ffmpeg_next::sys::AV_TIME_BASE as f32 * t) as i64;
                            if let Err(e) = input.seek(ts, ..ts) {
                                println!("DEBUG: failed when seek: {}", e);
                                continue;
                            }
                            decoder.flush();
                            seek_to = Some(t);
                        }
                    }
                    *event = DecoderEvent::None;
                }

                let mut buffer = Vec::with_capacity((w * h * 4) as usize);
                if frame_buf.is_none() {
                    // read packets
                    if let Some((stream, packet)) = input.packets().next() {
                        if stream.index() == video_ix {
                            // try to send packet to decoder
                            if decoder.send_packet(&packet).is_err() {
                                println!("DEBUG: error when send packet");
                                return;
                            }
                        } else if stream.index() == audio_ix {
                            // TODO: channel send
                        }
                    } else {
                        break;
                    };

                    // try receive decoder
                    if decoder.receive_frame(&mut decoded_frame).is_ok() {
                        // drop extra frames when seek
                        if let Some(to) = seek_to {
                            let target = (to * time_base.denominator() as f32) as i64;
                            if decoded_frame.pts().unwrap_or(0) < target {
                                continue;
                            } else {
                                seek_to = None;
                            }
                        }
                        // convert frame
                        scaler_context
                            .run(&decoded_frame, &mut scaled_frame)
                            .unwrap();

                        let data = scaled_frame.data(0);
                        let stride = scaled_frame.stride(0);

                        for y in 0..h as usize {
                            let start = y * stride;
                            let end = start + (w as usize * 4);
                            buffer.extend_from_slice(&data[start..end]);
                        }

                        frame_buf = Some(FrameImage {
                            image: generate_image_fallback(orignal_size, buffer),
                            pts: decoded_frame.pts().unwrap_or(0) as u64,
                        });
                    }
                }

                if producer.is_full() {
                    thread::sleep(Duration::from_millis(10));
                }
                if let Some(f) = frame_buf.take() {
                    if let Err(f) = producer.try_push(f) {
                        frame_buf = Some(f);
                    }
                }
            }
        });
    }
}
