use std::{
    io::{Cursor, Read, Write},
    os::fd::OwnedFd,
};

use pipewire::{self as pw, spa::param::video::VideoFormat};
use pw::{properties::properties, spa, spa::pod::Pod};
use tokio::sync::mpsc;
use x264::{Colorspace, Encoder, Image};

use crate::message::StreamMessage;

struct UserData {
    format: spa::param::video::VideoInfoRaw,
    tx: mpsc::Sender<StreamMessage>,
    encoder: Option<Encoder>,
    frame: u64,
}

pub async fn start_streaming(
    node_id: u32,
    fd: OwnedFd,
    tx: mpsc::Sender<StreamMessage>,
) -> Result<(), pw::Error> {
    pw::init();

    let mainloop = pw::main_loop::MainLoop::new(None)?;
    let context = pw::context::Context::new(&mainloop)?;
    let core = context.connect_fd(fd, None)?;

    let data = UserData {
        format: Default::default(),
        tx,
        encoder: None,
        frame: 0,
    };

    let stream = pw::stream::Stream::new(
        &core,
        "video-test",
        properties! {
            *pw::keys::MEDIA_TYPE => "Video",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Screen",
        },
    )?;

    let _listener = stream
        .add_local_listener_with_user_data(data)
        .state_changed(|_, _, old, new| {
            println!("State changed: {:?} -> {:?}", old, new);
        })
        .param_changed(|_, user_data, id, param| {
            let Some(param) = param else {
                return;
            };
            if id != pw::spa::param::ParamType::Format.as_raw() {
                return;
            }

            let (media_type, media_subtype) =
                match pw::spa::param::format_utils::parse_format(param) {
                    Ok(v) => v,
                    Err(_) => return,
                };

            if media_type != pw::spa::param::format::MediaType::Video
                || media_subtype != pw::spa::param::format::MediaSubtype::Raw
            {
                return;
            }

            user_data
                .format
                .parse(param)
                .expect("Failed to parse param changed to VideoInfoRaw");

            println!("got video format:");
            let video_format = user_data.format;
            println!(
                "  format: {} ({:?})",
                video_format.format().as_raw(),
                video_format.format()
            );
            println!(
                "  size: {}x{}",
                video_format.size().width,
                video_format.size().height
            );
            println!(
                "  framerate: {}/{}",
                video_format.framerate().num,
                video_format.framerate().denom
            );

            // Initialize the encoder
            let mut enc = Encoder::builder()
                .fps(30, 1)
                // .baseline()
                // FIXME: hardcoded colorspace
                .build(
                    Colorspace::BGR,
                    video_format.size().width.try_into().unwrap(),
                    video_format.size().height.try_into().unwrap(),
                )
                .unwrap();

            // Gen headers
            let headers = enc.headers().unwrap();
            let headers_data = Vec::from(headers.entirety());

            // Assign into UserData
            user_data.encoder = Some(enc);

            // Init message
            let tx_cloned = user_data.tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)); // gotta wait for server to be ready

                tx_cloned.send(StreamMessage::Ready).await.unwrap();
                tx_cloned
                    .send(StreamMessage::Header(headers_data))
                    .await
                    .unwrap();
            });
        })
        .process(|stream, user_data| {
            match stream.dequeue_buffer() {
                None => println!("out of buffers"),
                Some(mut buffer) => {
                    let datas = buffer.datas_mut();
                    if datas.is_empty() {
                        return;
                    }

                    // copy frame data to screen
                    let data = &mut datas[0];
                    let chunk = data.chunk();
                    // println!("got a frame of size {}", chunk.size());

                    let raw_data = match data.data() {
                        Some(data) => data,
                        None => return,
                    };

                    if user_data.format.format() != VideoFormat::BGRx {
                        eprintln!("unsupported pixel format: {:?}", user_data.format.format());
                        return;
                    };

                    let bgr_data = raw_data
                        .chunks_exact(4)
                        .flat_map(|chunk| [chunk[0], chunk[1], chunk[2]]) // Drop x
                        .collect::<Vec<_>>();

                    // Encode frame
                    let encoder = user_data.encoder.as_mut().expect("encoder unavailable");

                    let image = Image::bgr(encoder.width(), encoder.height(), &bgr_data); // faking the x part as alpha channel because BGRx isn't supported by x264
                    let (encoded_data, _) = encoder
                        .encode(user_data.frame.try_into().unwrap(), image)
                        .unwrap();
                    let encoded_data_vec = Vec::from(encoded_data.entirety());

                    // Update frame counter
                    user_data.frame += 1;

                    // Send frame to server
                    let tx_cloned = user_data.tx.clone();
                    let frame_cloned = user_data.frame.clone();
                    tokio::spawn(async move {
                        tx_cloned
                            .send(StreamMessage::Frame {
                                count: frame_cloned,
                                data: encoded_data_vec,
                            })
                            .await
                            .unwrap();
                    });
                }
            }
        })
        .register()?;

    println!("Created stream {:#?}", stream);

    let obj = pw::spa::pod::object!(
        pw::spa::utils::SpaTypes::ObjectParamFormat,
        pw::spa::param::ParamType::EnumFormat,
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaType,
            Id,
            pw::spa::param::format::MediaType::Video
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::MediaSubtype,
            Id,
            pw::spa::param::format::MediaSubtype::Raw
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            pw::spa::param::video::VideoFormat::RGB,
            pw::spa::param::video::VideoFormat::RGB,
            pw::spa::param::video::VideoFormat::RGBA,
            pw::spa::param::video::VideoFormat::RGBx,
            pw::spa::param::video::VideoFormat::BGRx,
            pw::spa::param::video::VideoFormat::YUY2,
            pw::spa::param::video::VideoFormat::I420,
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            pw::spa::utils::Rectangle {
                width: 320,
                height: 240
            },
            pw::spa::utils::Rectangle {
                width: 1,
                height: 1
            },
            pw::spa::utils::Rectangle {
                width: 4096,
                height: 4096
            }
        ),
        pw::spa::pod::property!(
            pw::spa::param::format::FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            pw::spa::utils::Fraction { num: 30, denom: 1 },
            pw::spa::utils::Fraction { num: 0, denom: 1 },
            pw::spa::utils::Fraction {
                num: 1000,
                denom: 1
            }
        ),
    );
    let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pw::spa::pod::Value::Object(obj),
    )
    .unwrap()
    .0
    .into_inner();

    let mut params = [Pod::from_bytes(&values).unwrap()];

    stream.connect(
        spa::utils::Direction::Input,
        Some(node_id),
        pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
        &mut params,
    )?;

    println!("Connected stream");

    mainloop.run();

    Ok(())
}
