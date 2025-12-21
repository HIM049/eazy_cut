pub fn calc_output_size(orignal_size: (u32, u32), view_size: (u32, u32)) -> Option<(u32, u32)> {
    if orignal_size == (0, 0) || view_size == (0, 0) {
        return None;
    }
    let orignal_width = orignal_size.0;
    let orignal_height = orignal_size.1;
    let view_width = view_size.0;
    let view_height = view_size.1;

    let scale_w = view_width as f32 / orignal_width as f32;
    let scale_h = view_height as f32 / orignal_height as f32;
    let scale = scale_w.min(scale_h);

    let out_width = (orignal_width as f32 * scale).round() as u32;
    let out_height = (orignal_height as f32 * scale).round() as u32;

    Some((out_width, out_height))
}
