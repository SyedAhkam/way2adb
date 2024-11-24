use ffmpeg_next::{
    self as ffmpeg,
    codec::{self, traits::Encoder},
    encoder, format, frame,
    software::scaling,
    Dictionary, Packet,
};

pub struct VideoEncoder {
    encoder: encoder::Video,
    scaler: scaling::Context,
    frame: usize,
}

impl VideoEncoder {
    pub fn new(width: u32, height: u32, fps: u32) -> Result<Self, Box<dyn std::error::Error>> {
        ffmpeg::init()?;

        // Create encoder
        let codec = encoder::find(codec::Id::H264).ok_or("H264 encoder not found")?;
        let mut context = codec::Context::new_with_codec(codec);
        let mut encoder = context.encoder().video()?;

        encoder.set_frame_rate(Some((30, 1)));
        encoder.set_width(width);
        encoder.set_height(height);
        encoder.set_time_base((1, fps.try_into()?));
        encoder.set_format(format::Pixel::YUV420P);
        // encoder.set_flags(codec::Flags::GLOBAL_HEADER);

        let mut opts = Dictionary::new();
        opts.set("preset", "veryfast");
        opts.set("tune", "zerolatency");
        opts.set("bitrate", "1000");
        opts.set("keyint", "30");
        opts.set(
            "x264-params",
            "nal-hrd=cbr:vbv-maxrate=1000:vbv-bufsize=1000",
        );
        opts.set("annexb", "1");
        opts.set("bframes", "0");

        let encoder = encoder.open_with(opts)?;

        // Create scaler for BGRx to YUV420P conversion
        let scaler = ffmpeg::software::scaling::Context::get(
            ffmpeg::format::Pixel::BGRZ,
            width,
            height,
            ffmpeg::format::Pixel::YUV420P,
            width,
            height,
            ffmpeg::software::scaling::Flags::BILINEAR,
        )?;

        Ok(Self {
            encoder,
            scaler,
            frame: 0,
        })
    }

    fn scale(
        &mut self,
        input_frame: frame::Video,
    ) -> Result<frame::Video, Box<dyn std::error::Error>> {
        let mut output_frame = frame::Video::empty();

        &self.scaler.run(&input_frame, &mut output_frame)?;

        Ok(output_frame)
    }

    pub fn encode(&mut self, bgrx_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut input_frame = frame::Video::new(
            format::Pixel::BGRZ,
            self.encoder.width(),
            self.encoder.height(),
        );
        input_frame.data_mut(0).copy_from_slice(bgrx_data);

        let mut yuv_frame = self.scale(input_frame)?;

        yuv_frame.set_pts(Some(self.frame as i64));
        self.frame += 1;

        let mut encoded_packets = Vec::new();
        self.encoder.send_frame(&yuv_frame)?;

        let mut packet = Packet::empty();
        while let Ok(_) = self.encoder.receive_packet(&mut packet) {
            encoded_packets.extend_from_slice(packet.data().unwrap())
        }

        Ok(encoded_packets)
    }

    pub fn get_frame(&self) -> usize {
        self.frame
    }
}
