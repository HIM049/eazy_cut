use std::{sync::Arc, time::Instant};

use gpui::{Context, Entity, RenderImage, Window};
use ringbuf::{
    HeapCons,
    storage::Heap,
    traits::{Consumer, Split},
};

use crate::ui::{
    app::MyApp,
    player::{
        ffmpeg::VideoDecoder, frame::FrameImage, player_size::PlayerSize,
        utils::generate_image_fallback, viewer::Viewer,
    },
};

pub struct Player {
    size: Entity<PlayerSize>,
    decoder: VideoDecoder,
    frame: Arc<RenderImage>,
    frame_buf: Option<FrameImage>,
    consumer: HeapCons<FrameImage>,
    start_time: Option<Instant>,
}

impl Player {
    pub fn new(size_entity: Entity<PlayerSize>) -> Self {
        let rb = ringbuf::SharedRb::<Heap<FrameImage>>::new(30 * 1);
        let (producer, consumer) = rb.split();
        Self {
            size: size_entity.clone(),
            decoder: VideoDecoder::new(size_entity).set_producer(producer),
            frame: generate_image_fallback((1, 1), vec![]),
            frame_buf: None,
            consumer,
            start_time: None,
        }
    }

    pub fn open(&mut self, cx: &mut Context<MyApp>) {
        self.decoder
            .open(
                cx,
                "D:/Videos/Records/Apex Legends 2024.05.04 - 18.07.10.04.DVR.mp4".into(),
            )
            .unwrap();
    }

    pub fn start_play(&mut self, cx: &mut Context<MyApp>) {
        self.decoder.spawn_decoder(self.size.clone(), cx);
    }

    fn compare_time(&mut self, frame_pts: u64) -> bool {
        if self.start_time.is_none() {
            self.start_time = Some(std::time::Instant::now());
        }
        let Some(time) = self.start_time else {
            return false;
        };
        let Some(time_base) = self.decoder.get_timebase() else {
            return false;
        };

        let elapsed = time.elapsed().as_secs_f32();
        let frame_time = frame_pts as f32 / time_base.denominator() as f32;

        if frame_time <= elapsed {
            println!(
                "frame_time: {:6.2} | time: {:6.2} | frame_pts: {}",
                frame_time, elapsed, frame_pts
            );
        }

        frame_time <= elapsed
    }

    pub fn view(&mut self, w: &mut Window) -> Viewer {
        if let Some(fb) = self.frame_buf.take() {
            if self.compare_time(fb.pts) {
                w.drop_image(self.frame.clone()).unwrap();
                self.frame = fb.image;
            } else {
                self.frame_buf = Some(fb);
            }
        } else {
            if let Some(f) = self.consumer.try_pop() {
                if self.compare_time(f.pts) {
                    w.drop_image(self.frame.clone()).unwrap();
                    self.frame = f.image;
                } else {
                    self.frame_buf = Some(f);
                }
            }
        }
        Viewer::new(self.frame.clone(), self.size.clone())
    }
}
