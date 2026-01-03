use std::path::PathBuf;

pub fn output(path: &PathBuf, stream_ix: usize, time_range: (f32, f32)) {
    println!(
        "DEBUG: run output, path: {:?}, stream_ix: {}, time_range: {:?}",
        path, stream_ix, time_range
    );
    let mut input = ffmpeg_next::format::input(&path).unwrap();
    let ts = (ffmpeg_next::sys::AV_TIME_BASE as f32 * time_range.0) as i64;
    input.seek(ts, ..ts).unwrap();
    let stream = input.stream(stream_ix).unwrap();

    let mut output = ffmpeg_next::format::output("./output.mp4").unwrap();
    {
        let mut out_stream = output.add_stream(None).unwrap();
        out_stream.set_parameters(stream.parameters());
    }

    output.write_header().unwrap();
    let out_tb = output.stream(0).unwrap().time_base();

    let mut offset_pts: Option<i64> = None;
    let mut offset_dts: Option<i64> = None;
    for (stream, mut packet) in input.packets() {
        if offset_pts.is_none() {
            offset_pts = packet.pts();
            offset_dts = packet.dts();
        }
        let pkt_pts = packet.pts().unwrap_or(0);
        let pkt_dts = packet.dts().unwrap_or(0);
        let frame_time = pkt_pts as f32 / stream.time_base().denominator() as f32;
        // when out the range
        if frame_time > time_range.1 {
            break;
        }
        if stream.index() == stream_ix {
            packet.set_pts(Some(pkt_pts - offset_pts.unwrap()));
            packet.set_dts(Some(pkt_dts - offset_dts.unwrap()));

            packet.rescale_ts(stream.time_base(), out_tb);
            packet.set_position(-1);
            packet.set_stream(0);
            packet.write_interleaved(&mut output).unwrap();
        }
    }

    output.write_trailer().unwrap();
}
