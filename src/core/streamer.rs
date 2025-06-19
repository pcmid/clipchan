use std::sync::Arc;

use anyhow::anyhow;
use futures_util::StreamExt;
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;
use serde::Deserialize;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::select;
use tokio::sync::{Mutex, Notify};
use tracing;

#[derive(Debug, Clone, Deserialize)]
pub struct RtmpStreamerConfig {
    // video
    pub video_height: i32,
    pub video_width: i32,
    pub video_framerate: i32,
    pub video_bitrate: u32,
    pub video_speed: String, // "ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"

    // audio
    pub audio_sample_format: String,
    pub audio_rate: i32,
    pub audio_channels: i32,
    pub audio_bitrate: i32,

    pub title_font: String,
    pub title_halign: String,
    pub title_valign: String,
}

impl Default for RtmpStreamerConfig {
    fn default() -> Self {
        Self {
            video_height: 720,
            video_width: 1280,
            video_framerate: 30,
            video_bitrate: 2500, // in kbps
            video_speed: "faster".to_string(),
            audio_sample_format: "S16LE".to_string(),
            audio_rate: 44100,
            audio_channels: 2,
            audio_bitrate: 128_000, // in bps
            title_font: "Sans, 24".to_string(),
            title_halign: "right".to_string(),
            title_valign: "top".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct RtmpStreamer {
    config: RtmpStreamerConfig,
    state: Arc<Mutex<StreamerState>>,
    pub rtmp_url: String,
}

struct StreamerState {
    video_src: Option<gst::Element>,
    audio_src: Option<gst::Element>,
    title: Option<gst::Element>,
    pipeline: Option<gst::Pipeline>,
    stop: Option<Arc<Notify>>,
    is_streaming: bool,
}

impl RtmpStreamer {
    pub fn new(config: RtmpStreamerConfig, rtmp_url: &str) -> anyhow::Result<Self> {
        gst::init()?;
        Ok(Self {
            config,
            rtmp_url: rtmp_url.to_string(),
            state: Arc::new(Mutex::new(StreamerState {
                video_src: None,
                audio_src: None,
                title: None,
                pipeline: None,
                stop: None,
                is_streaming: false,
            })),
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let pipeline = gst::Pipeline::new();
        // video
        let video_src = gst::ElementFactory::make("appsrc")
            .name("videosrc")
            .property("is-live", &true)
            .property("do-timestamp", &true)
            .property("format", &gst::Format::Time)
            .build()?;

        let videoconvert = gst::ElementFactory::make("videoconvert").build()?;
        let overlay = gst::ElementFactory::make("textoverlay")
            .name("overlay")
            .property("text", "Initial Title")
            .property_from_str("halignment", &self.config.title_halign)
            .property_from_str("valignment", &self.config.title_valign)
            .property_from_str("font-desc", &self.config.title_font)
            .property("shaded-background", &true)
            .build()?;
        let x264enc = gst::ElementFactory::make("x264enc")
            .property_from_str("tune", &"zerolatency")
            .property("bitrate", &self.config.video_bitrate)
            .property("key-int-max", &30u32)
            .property("bframes", &0u32)
            .property("ref", &2u32)
            .property("byte-stream", &true)
            .property("vbv-buf-capacity", &0u32)
            .property_from_str("speed-preset", &self.config.video_speed)
            .build()?;
        let h264parse = gst::ElementFactory::make("h264parse").build()?;
        let h264_caps = gst::Caps::builder("video/x-h264")
            .field("profile", &"main")
            .build();
        let capsfilter_h264 = gst::ElementFactory::make("capsfilter")
            .property("caps", &h264_caps)
            .build()?;

        // video
        let audio_src = gst::ElementFactory::make("appsrc")
            .name("audiosrc")
            .property("is-live", &true)
            .property("do-timestamp", &true)
            .property("format", &gst::Format::Time)
            .build()?;
        let audioconvert = gst::ElementFactory::make("audioconvert").build()?;

        let aacenc = gst::ElementFactory::make("fdkaacenc").build()?;
        let aacparse = gst::ElementFactory::make("aacparse").build()?;
        let aac_caps = gst::Caps::builder("audio/mpeg")
            .field("mpegversion", 4i32)
            .field("stream-format", "raw")
            .field("bitrate", &self.config.audio_bitrate)
            .build();
        let capsfilter_aac = gst::ElementFactory::make("capsfilter")
            .property("caps", &aac_caps)
            .build()?;

        // rtmp muxer and sink
        let muxer = gst::ElementFactory::make("flvmux")
            .name("mux")
            .property("streamable", &true)
            .property("latency", &(15 * 1000u64))
            .build()?;
        let queue_muxer = gst::ElementFactory::make("queue")
            .name("muxer_queue")
            .property_from_str("leaky", "no")
            .property("max-size-buffers", &900u32)
            .property("max-size-time", &15_000_000_000u64)
            .build()?;
        let sink = gst::ElementFactory::make("rtmpsink")
            .property("location", &self.rtmp_url)
            .property("sync", &false)
            .build()?;

        // add elements to pipeline
        pipeline.add_many(&[
            &video_src,
            &videoconvert,
            &overlay,
            &x264enc,
            &h264parse,
            &capsfilter_h264,
            &audio_src,
            &audioconvert,
            &aacenc,
            &aacparse,
            &capsfilter_aac,
            &muxer,
            &queue_muxer,
            &sink,
        ])?;

        // link video elements
        gst::Element::link_many(&[
            &video_src,
            &videoconvert,
            &overlay,
            &x264enc,
            &h264parse,
            &capsfilter_h264,
            &muxer,
        ])?;

        // link audio elements
        gst::Element::link_many(&[
            &audio_src,
            &audioconvert,
            &aacenc,
            &aacparse,
            &capsfilter_aac,
            &muxer,
        ])?;
        muxer.link(&queue_muxer)?;
        queue_muxer.link(&sink)?;

        let mut state = self.state.lock().await;
        if state.is_streaming {
            return Err(anyhow!("A push pipeline is already running"));
        }

        if state.pipeline.is_some() {
            return Err(anyhow!("A push pipeline is already running"));
        }

        let pipeline_clone = pipeline.clone();
        let bus = pipeline
            .bus()
            .ok_or_else(|| anyhow!("Failed to get bus from pipeline"))?;
        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();
        tokio::spawn(async move {
            let mut messages = bus.stream();
            select! {
                _ = notify_clone.notified() => {
                    tracing::debug!("Push pipeline notification received, stopping message processing");
                    return;
                }
                Some(msg) = messages.next() => {
                    match msg.view() {
                    gst::MessageView::Eos(_) => {
                        tracing::debug!("Main pipeline EOS received");
                        notify_clone.notify_one();
                        return;
                    }
                    gst::MessageView::Error(err) => {
                        tracing::error!(
                            "Main pipeline error: {}: {}",
                            err.error(),
                            err.debug().unwrap_or("No debug info available".into()),
                        );
                        return
                    }
                    gst::MessageView::StateChanged(state) => {
                        state.src().map(|s| {
                            if s == pipeline_clone.upcast_ref::<gst::Object>() {
                                let old = state.old();
                                let new = state.current();
                                tracing::trace!(
                                    "Main pipeline state changed: {:?} -> {:?} (pending: {:?})",
                                    old,
                                    new,
                                    state.pending()
                                );
                            }
                        });
                    }
                    _ => {
                        tracing::trace!("Unhandled message: {:?}", msg);
                        }
                }
                }
            }
        });

        pipeline.set_state(gst::State::Playing)?;
        state.pipeline = Some(pipeline.clone());
        state.video_src = Some(video_src);
        state.audio_src = Some(audio_src);
        state.title = Some(overlay);
        state.stop = Some(notify);
        state.is_streaming = true;

        Ok(())
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        tracing::trace!("Stopping push pipeline");
        let mut state = self.state.lock().await;
        if let Some(pipeline) = state.pipeline.take() {
            pipeline.set_state(gst::State::Null)?;
            let (result, current, pending) = pipeline.state(gst::ClockTime::from_seconds(5));
            result.map_err(|e| {
                tracing::error!("Failed to set pipeline to Null state: {}, current state: {:?}, pending state: {:?}", e, current, pending);
                anyhow!("Failed to set pipeline to Null state: {}", e)
            })?;
            state.title = None;
            state.is_streaming = false;
            state.stop.take_if(|n| {
                n.notify_waiters();
                true
            });
            tracing::debug!("Push pipeline stopped successfully");
        } else {
            return Err(anyhow!("No active push pipeline to stop"));
        }
        Ok(())
    }

    pub async fn update_title(&self, new_title: &str) -> anyhow::Result<()> {
        tracing::trace!("Starting to update title to: {}", new_title);
        let state = self.state.lock().await;
        if let Some(overlay) = &state.title {
            overlay.set_property("text", &new_title);
        }
        tracing::debug!("Updated title: {}", new_title);
        Ok(())
    }

    pub async fn push<R>(&self, reader: R) -> anyhow::Result<()>
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        // Create decoding pipeline
        let pipeline = gst::Pipeline::new();

        // Create source element
        let src = gst::ElementFactory::make("appsrc")
            .name("source")
            .property("is-live", &true)
            .property("format", &gst::Format::Time)
            .property("block", &true)
            .property("max-bytes", &65536u64)
            .property("do-timestamp", &true)
            .build()?;
        let decodebin = gst::ElementFactory::make("decodebin").build()?;
        pipeline.add_many(&[&src, &decodebin])?;
        src.link(&decodebin)?;

        // Create video and audio converters and sinks
        let video_convert = gst::ElementFactory::make("videoconvert").build()?;
        let video_rate = gst::ElementFactory::make("videorate").build()?;
        let video_scale = gst::ElementFactory::make("videoscale").build()?;
        let video_caps = gst::Caps::builder("video/x-raw")
            .field("format", &"I420")
            .field("width", &self.config.video_width)
            .field("height", &self.config.video_height)
            .field(
                "framerate",
                &gst::Fraction::new(self.config.video_framerate, 1),
            )
            .build();
        let video_capsfilter = gst::ElementFactory::make("capsfilter")
            .property("caps", &video_caps)
            .build()?;
        let video_sink = gst::ElementFactory::make("appsink")
            .name("video_sink")
            .property("sync", &true)
            .property("emit-signals", &true)
            .property("drop", &true)
            .build()?;

        pipeline.add_many(&[
            &video_convert,
            &video_rate,
            &video_scale,
            &video_capsfilter,
            &video_sink,
        ])?;
        gst::Element::link_many(&[
            &video_convert,
            &video_rate,
            &video_scale,
            &video_capsfilter,
            &video_sink,
        ])?;

        let audio_convert = gst::ElementFactory::make("audioconvert").build()?;
        let audio_rate = gst::ElementFactory::make("audiorate").build()?;
        let audio_resample = gst::ElementFactory::make("audioresample").build()?;

        let audio_caps = gst::Caps::builder("audio/x-raw")
            .field("format", &self.config.audio_sample_format)
            .field("rate", &self.config.audio_rate)
            .field("channels", &self.config.audio_channels)
            .field("layout", &"interleaved")
            .build();
        let audio_capsfilter = gst::ElementFactory::make("capsfilter")
            .property("caps", &audio_caps)
            .build()?;
        let audio_sink = gst::ElementFactory::make("appsink")
            .name("audio_sink")
            .property("sync", &true)
            .property("emit-signals", &true)
            .property("drop", &true)
            .build()?;

        pipeline.add_many(&[
            &audio_convert,
            &audio_rate,
            &audio_resample,
            &audio_capsfilter,
            &audio_sink,
        ])?;
        gst::Element::link_many(&[
            &audio_convert,
            &audio_rate,
            &audio_resample,
            &audio_capsfilter,
            &audio_sink,
        ])?;

        // Convert to AppSink type
        let video_sink = video_sink
            .clone()
            .dynamic_cast::<gst_app::AppSink>()
            .unwrap();
        let audio_sink = audio_sink
            .clone()
            .dynamic_cast::<gst_app::AppSink>()
            .unwrap();
        let audio_convert_clone = audio_convert.clone();
        let video_convert_clone = video_convert.clone();

        // Handle new pads
        decodebin.connect_pad_added(move |_bin, pad| {
            let caps = pad.current_caps().unwrap();
            let name = caps.structure(0).unwrap().name();
            tracing::trace!("Decoder new pad: {} type: {}", pad.name(), name);

            if name.starts_with("audio/") {
                let sink_pad = audio_convert_clone.static_pad("sink").unwrap();
                if !sink_pad.is_linked() {
                    match pad.link(&sink_pad) {
                        Ok(_) => tracing::debug!("Audio pad linked successfully"),
                        Err(e) => tracing::error!("Audio pad linking failed: {}", e),
                    }
                }
            } else if name.starts_with("video/") {
                let sink_pad = video_convert_clone.static_pad("sink").unwrap();
                if !sink_pad.is_linked() {
                    match pad.link(&sink_pad) {
                        Ok(_) => tracing::debug!("Video pad linked successfully"),
                        Err(e) => tracing::error!("Video pad linking failed: {}", e),
                    }
                }
            } else {
                tracing::warn!("Unknown media type: {}", name);
            }
        });

        // Get video and audio destination sources
        let state = self.state.lock().await;
        let video_dst = state
            .video_src
            .clone()
            .ok_or(anyhow!("Video source not initialized"))?
            .dynamic_cast::<gst_app::AppSrc>()
            .unwrap();
        let audio_dst = state
            .audio_src
            .clone()
            .ok_or(anyhow!("Audio source not initialized"))?
            .dynamic_cast::<gst_app::AppSrc>()
            .unwrap();
        video_dst.set_property("do-timestamp", &true);
        audio_dst.set_property("do-timestamp", &true);
        // Set audio sink callbacks
        audio_sink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample({
                    let audio_dst = audio_dst.clone();
                    move |sink| {
                        let sample = sink.pull_sample().map_err(|_| {
                            tracing::error!("Failed to get sample from audio sink");
                            gst::FlowError::Eos
                        })?;

                        let mut buffer = sample.buffer_owned().ok_or(gst::FlowError::Error)?;
                        let buffer_ref = buffer.make_mut();
                        buffer_ref.set_pts(gst::ClockTime::NONE);
                        buffer_ref.set_dts(gst::ClockTime::NONE);

                        let caps = sample.caps().map(|c| c.copy());
                        let segment = sample.segment();
                        let sample = gst::Sample::builder()
                            .buffer(&buffer)
                            .caps_if_some(caps.as_ref())
                            .segment_if_some(segment)
                            .build();

                        audio_dst.push_sample(&sample).map_err(|e| {
                            if e == gst::FlowError::Flushing {
                                tracing::debug!("Audio source Flushing, stopping push");
                                return gst::FlowError::Eos;
                            }
                            tracing::error!("Failed to push sample to audio source: {}", e);
                            e
                        })
                    }
                })
                .build(),
        );

        // Set video sink callbacks
        video_sink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample({
                    let video_dst = video_dst.clone();
                    move |sink| {
                        let sample = sink.pull_sample().map_err(|_| {
                            tracing::error!("Failed to get sample from video sink");
                            gst::FlowError::Eos
                        })?;

                        let mut buffer = sample.buffer_owned().ok_or(gst::FlowError::Error)?;
                        let buffer_ref = buffer.make_mut();
                        buffer_ref.set_pts(gst::ClockTime::NONE);
                        buffer_ref.set_dts(gst::ClockTime::NONE);

                        let caps = sample.caps().map(|c| c.copy());
                        let segment = sample.segment();
                        let sample = gst::Sample::builder()
                            .buffer(&buffer)
                            .caps_if_some(caps.as_ref())
                            .segment_if_some(segment)
                            .build();

                        video_dst.push_sample(&sample).map_err(|e| {
                            if e == gst::FlowError::Flushing {
                                tracing::debug!("Video source Flushing, stopping push");
                                return gst::FlowError::Eos;
                            }
                            tracing::error!("Failed to push sample to video source: {}", e);
                            e
                        })
                    }
                })
                .build(),
        );
        drop(state);

        // Push input stream to source
        tracing::debug!("Starting to push stream");
        let src = src.clone().dynamic_cast::<gst_app::AppSrc>().unwrap();
        let mut buf = [0u8; 4096];
        let state = self.state.clone();
        tokio::spawn(async move {
            tracing::trace!("Starting to read data from async reader");
            let mut reader = reader;
            while let Ok(n) = reader.read(&mut buf).await {
                if n == 0 {
                    tracing::trace!("Input stream ended");
                    break;
                }

                let gst_buf = gst::Buffer::from_slice(buf[..n].to_vec());
                let state = state.lock().await;
                if !state.is_streaming {
                    tracing::warn!("Pipeline not streaming, stopping buffer push");
                    break;
                }
                src.push_buffer(gst_buf)
                    .inspect_err(|err| {
                        tracing::warn!("Failed to push buffer to appsrc: {}", err);
                    })
                    .ok();
            }
            tracing::trace!("Sending EOS to appsrc");
            src.end_of_stream().ok();
        });

        // Monitor pipeline events
        let pipeline_clone = pipeline.clone();
        let bus = pipeline
            .bus()
            .ok_or_else(|| anyhow!("Failed to get pipeline bus"))?;
        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();
        tokio::spawn(async move {
            let mut messages = bus.stream();
            while let Some(msg) = messages.next().await {
                match msg.view() {
                    gst::MessageView::Eos(_) => {
                        tracing::debug!("Decoding pipeline received EOS");
                        notify_clone.notify_one();
                        break;
                    }
                    gst::MessageView::Error(err) => {
                        tracing::error!(
                            "Decoding pipeline error: {}: {}",
                            err.error(),
                            err.debug().unwrap_or("No debug info available".into()),
                        );
                        notify_clone.notify_one();
                        break;
                    }
                    gst::MessageView::StateChanged(state) => {
                        state.src().map(|s| {
                            if s == pipeline_clone.upcast_ref::<gst::Object>() {
                                let old = state.old();
                                let new = state.current();
                                tracing::trace!(
                                    "Decoding pipeline state changed: {:?} -> {:?} (pending: {:?})",
                                    old,
                                    new,
                                    state.pending()
                                );
                            }
                        });
                    }
                    _ => {}
                }
            }
        });

        // Start pipeline
        pipeline.set_state(gst::State::Playing)?;
        let (_change_success, current_state, pending_state) =
            pipeline.state(gst::ClockTime::from_seconds(5));
        if current_state != gst::State::Playing {
            return Err(anyhow!(
                "Failed to set decoding pipeline to Playing state: {:?} (pending: {:?})",
                current_state,
                pending_state
            ));
        }

        // Wait for pipeline to complete
        notify.notified().await;
        tracing::debug!("Decoding pipeline {} has stopped", pipeline.name());
        pipeline.set_state(gst::State::Null)?;
        let (result, current, pending) = pipeline.state(gst::ClockTime::from_seconds(5));
        result.map_err(|e| {
            tracing::error!("Failed to set pipeline to Null state: {}, current state: {:?}, pending state: {:?}", e, current, pending);
            anyhow!("Failed to set pipeline to Null state: {}", e)
        })?;
        Ok(())
    }

    pub async fn _is_streaming(&self) -> bool {
        let state = self.state.lock().await;
        state.is_streaming
    }
}

impl Drop for RtmpStreamer {
    fn drop(&mut self) {
        if let Ok(state) = self.state.try_lock() {
            if let Some(pipeline) = &state.pipeline {
                tracing::trace!("RtmpStreamer being destroyed, stopping all streams");
                pipeline
                    .set_state(gst::State::Null)
                    .inspect_err(|e| {
                        tracing::error!("Failed to stop streaming: {}", e);
                    })
                    .ok();
            }
        } else {
            tracing::warn!(
                "Failed to acquire lock on RtmpStreamer state during drop, this may lead to resource leaks"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use ctor::ctor;
    use tokio::fs::File;

    use super::*;

    #[ctor]
    fn init_tracing() {
        tracing_subscriber::fmt().with_env_filter("trace").init();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_rtmp_streamer() -> anyhow::Result<()> {
        let config = RtmpStreamerConfig::default();
        let streamer = RtmpStreamer::new(config, "rtmp://127.0.0.1:1935/live/test")?;

        // Start the pipeline with a test title
        streamer.start().await?;

        // read from test files
        let video_file = File::open("data/test_001.mp4").await?;
        streamer.update_title("Test test_000 Stream").await?;
        streamer.push(video_file).await?;

        let video_file = File::open("data/test_001.mp4").await?;
        streamer.update_title("Test test_001 Stream").await?;
        streamer.push(video_file).await?;

        let video_file = File::open("data/test_002.mp4").await?;
        streamer.update_title("Test test_002 Stream").await?;
        streamer.push(video_file).await?;

        drop(streamer);
        Ok(())
    }
}
