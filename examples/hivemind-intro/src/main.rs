use fframes::{
    AudioMixOptions, CombinedMediaProvider, EncoderOptions, LimiterOptions, MediaDirectory,
    MediaProvider, RenderOptions, StaticMediaProvider, cli,
};
use fframes::ffmpeg_sys_fframes::AVPixelFormat;
use fframes_skia_renderer::{SkiaPipelineConcurrencyPolicy, SkiaPipelineConfig, cli::SkiaRenderers};
use hivemind_intro::{IntroMedia, IntroVideo, takes_dir};
use std::process::ExitCode;

fn main() -> ExitCode {
    let media = IntroMedia::prepare().expect("media");
    let folder = takes_dir();
    let dir = MediaDirectory::read_folder(&folder).expect("the takes folder (run scripts/prep.sh)");
    let dynamic = dir.process_media_source().expect("takes");
    let all = CombinedMediaProvider::from([&media as &dyn MediaProvider, &dynamic]);
    let prores: [(&str, &str); 1] = [("profile", "3")];

    cli::new(
        &IntroVideo,
        RenderOptions {
            media: Some(&all),
            load_system_fonts: false,
            default_font: "Geist",
            video_encoder_options: EncoderOptions {
                preferred_encoder: Some("prores_ks"),
                pixel_format: AVPixelFormat::AV_PIX_FMT_YUV422P10LE,
                codec_params: Some(&prores),
                ..Default::default()
            },
            audio_mix: AudioMixOptions {
                limiter: Some(LimiterOptions { ceiling_db: -1.5, ..Default::default() }),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .default_output("build/intro.mov")
    .renderers(SkiaRenderers::gpu(SkiaPipelineConfig {
        concurrency_policy: SkiaPipelineConcurrencyPolicy::MaxPerformance,
        ..Default::default()
    }))
    .run()
}
