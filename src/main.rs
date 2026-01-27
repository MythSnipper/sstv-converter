use std::env;
use std::str::FromStr;
use std::f32::consts::PI;

use image::GenericImageView;
use image::imageops::FilterType;

#[derive(Debug)]
enum SSTVMode {
    R12,
    R24,
    R36,
    R72,
    M1,
    M2,
    M3,
    M4,
    S1,
    S2,
    S3,
    S4,
    SDX,
}

impl SSTVMode {
    fn resolution(&self) -> (u32, u32) {
        match self {
            SSTVMode::R12 => (160, 120),
            SSTVMode::R24 => (320, 120),
            SSTVMode::R36 => (320, 240),
            SSTVMode::R72 => (320, 240),
            SSTVMode::M1 => (320, 256),
            SSTVMode::M2 => (160, 256),
            SSTVMode::M3 => (320, 128),
            SSTVMode::M4 => (160, 128),
            SSTVMode::S1 => (320, 256),
            SSTVMode::S2 => (160, 256),
            SSTVMode::S3 => (320, 128),
            SSTVMode::S4 => (160, 128),
            SSTVMode::SDX => (320, 256),
        }
    }
    fn vis_code(&self) -> u8 {
        match self {
            SSTVMode::M1 => 0b0101100,
            SSTVMode::M2 => 0b0101000,
            SSTVMode::M3 => 0b0100100,
            SSTVMode::M4 => 0b0100000,
            SSTVMode::S1 => 0b0111100,
            SSTVMode::S2 => 0b0111000,
            SSTVMode::S3 => 0b0110100,
            SSTVMode::S4 => 0b0110000,
            SSTVMode::SDX => 0b1001100,
        }
    }
    fn color_scanline_ms(&self) -> f32 {
        match self {
            SSTVMode::M1 => 146.432,
            SSTVMode::M2 => 73.216,
            SSTVMode::M3 => 146.432,
            SSTVMode::M4 => 73.216,
            SSTVMode::S1 => 138.240,
            SSTVMode::S2 => 88.064,
            SSTVMode::S3 => 138.240,
            SSTVMode::S4 => 88.064,
            SSTVMode::SDX => 345.600,
        }
    }
    fn write_scanlines<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut hound::WavWriter<W>,
        osc: &mut Oscillator,
        image: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>
        ) {
        match self {
            SSTVMode::M1 | SSTVMode::M2 | SSTVMode::M3 | SSTVMode::M4 => {
                let width = self.resolution().0 as usize;
                let height = self.resolution().1 as usize;

                const LINE_SYNC_HZ: f32 = 1200.0;
                const SEP_HZ: f32 = 1500.0;

                const LINE_SYNC_MS: f32 = 4.862;
                let color_scan_ms = self.color_scanline_ms();
                const SEP_MS: f32 = 0.572;

                let pixel_ms = color_scan_ms / width as f32;

                for y in 0..height {
                    //line sync
                    emit_tone(writer, osc, LINE_SYNC_HZ, LINE_SYNC_MS);
                    //separator
                    emit_tone(writer, osc, SEP_HZ, SEP_MS);
                    //green
                    for x in 0..width {
                        let pixel = image.get_pixel(x as u32, y as u32);
                        let g = pixel[1] as f32 / 255.0;
                        let freq = 1500.0 + (2300.0 - 1500.0) * g;
                        emit_tone(writer, osc, freq, pixel_ms);
                    }
                    //separator
                    emit_tone(writer, osc, SEP_HZ, SEP_MS);

                    //blue
                    for x in 0..width {
                        let pixel = image.get_pixel(x as u32, y as u32);
                        let b = pixel[2] as f32 / 255.0;
                        let freq = 1500.0 + (2300.0 - 1500.0) * b;
                        emit_tone(writer, osc, freq, pixel_ms);
                    }
                    //separator
                    emit_tone(writer, osc, SEP_HZ, SEP_MS);
                    //red
                    for x in 0..width {
                        let pixel = image.get_pixel(x as u32, y as u32);
                        let r = pixel[0] as f32 / 255.0;
                        let freq = 1500.0 + (2300.0 - 1500.0) * r;
                        emit_tone(writer, osc, freq, pixel_ms);
                    }
                    //separator
                    emit_tone(writer, osc, SEP_HZ, SEP_MS);
                }
            }
            SSTVMode::S1 | SSTVMode::S2 | SSTVMode::S3 | SSTVMode::S4 | SSTVMode::SDX => {
                let width = self.resolution().0 as usize;
                let height = self.resolution().1 as usize;

                const LINE_SYNC_HZ: f32 = 1200.0;
                const SEP_HZ: f32 = 1500.0;

                const LINE_SYNC_MS: f32 = 9.0;
                let color_scan_ms = self.color_scanline_ms();
                const SEP_MS: f32 = 1.5;

                let pixel_ms = color_scan_ms / width as f32;

                for y in 0..height {

                    //separator
                    emit_tone(writer, osc, SEP_HZ, SEP_MS);

                    //green
                    for x in 0..width {
                        let pixel = image.get_pixel(x as u32, y as u32);
                        let g = pixel[1] as f32 / 255.0;
                        let freq = 1500.0 + (2300.0 - 1500.0) * g;
                        emit_tone(writer, osc, freq, pixel_ms);
                    }

                    //separator
                    emit_tone(writer, osc, SEP_HZ, SEP_MS);

                    //blue
                    for x in 0..width {
                        let pixel = image.get_pixel(x as u32, y as u32);
                        let b = pixel[2] as f32 / 255.0;
                        let freq = 1500.0 + (2300.0 - 1500.0) * b;
                        emit_tone(writer, osc, freq, pixel_ms);
                    }

                    //line sync
                    emit_tone(writer, osc, LINE_SYNC_HZ, LINE_SYNC_MS);

                    //separator
                    emit_tone(writer, osc, SEP_HZ, SEP_MS);

                    //red
                    for x in 0..width {
                        let pixel = image.get_pixel(x as u32, y as u32);
                        let r = pixel[0] as f32 / 255.0;
                        let freq = 1500.0 + (2300.0 - 1500.0) * r;
                        emit_tone(writer, osc, freq, pixel_ms);
                    }

                }
            }



        }
    }
}

impl FromStr for SSTVMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "M1" | "Martin1" => Ok(SSTVMode::M1),
            "M2" | "Martin2" => Ok(SSTVMode::M2),
            "M3" | "Martin3" => Ok(SSTVMode::M3),
            "M4" | "Martin4" => Ok(SSTVMode::M4),
            "S1" | "Scottie1" => Ok(SSTVMode::S1),
            "S2" | "Scottie2" => Ok(SSTVMode::S2),
            "S3" | "Scottie3" => Ok(SSTVMode::S3),
            "S4" | "Scottie4" => Ok(SSTVMode::S4),
            "SDX" | "ScottieDX" => Ok(SSTVMode::SDX),
            _ => Err(format!("Unknown SSTV mode: {}", s)),
        }
    }
}



pub struct Oscillator {
    pub sample_rate: u32,
    phase: f32,
    frac_samples: f32,
    pub amplitude: f32,
}

impl Oscillator {
    pub fn new(sample_rate: u32, amplitude: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            frac_samples: 0.0,
            amplitude: amplitude.clamp(0.0, 1.0),
        }
    }
}


fn main(){
    let mut argv: Vec<String> = env::args().collect();

    //break down argv
    let mut sstv_mode: SSTVMode = SSTVMode::S1;

    let mut volume: f32 = 50.0;

    let mut sample_rate: u32 = 44100;

    let mut infile_path: String = String::from("");
    let mut outfile_path: String = String::from("out.wav");

    parse_args(&mut argv, &mut sstv_mode, &mut volume, &mut sample_rate, &mut infile_path, &mut outfile_path);

    println!("Mode: {:?}", sstv_mode);
    println!("Volume: {}%", volume);
    println!("Sample rate: {} Hz", sample_rate);
    println!("Infile: {}", infile_path);
    println!("Outfile: {}", outfile_path);

    //load image
    let image = image::open(infile_path)
        .expect("Failed to open image");
    let image_resolution = image.dimensions();
    //to rgb8
    let image = image.to_rgb8();
    //resize to target resolution
    let target_resolution = sstv_mode.resolution(); 
    let image = image::imageops::resize(
        &image,
        target_resolution.0,
        target_resolution.1,
        FilterType::Lanczos3
    );
    println!("Image resized from {}x{} to {}x{}", image_resolution.0, image_resolution.1, target_resolution.0, target_resolution.1);

    //make wav file
    let spec = hound::WavSpec{
        channels: 1,
        sample_rate: sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };

    let mut writer = hound::WavWriter::create(outfile_path, spec)
        .expect("Failed to create wav file");

    let mut osc = Oscillator::new(sample_rate, volume);

    write_vis(&mut writer, &mut osc, sstv_mode.vis_code());

    //sync and write image scanlines
    println!("Writing image scanlines");

    sstv_mode.write_scanlines(&mut writer, &mut osc, &image);

    writer.finalize().unwrap();

    println!("Done");

}

/*
usage: ffmpeg [options] [[infile options] -i infile]... {[outfile options] outfile}...

Getting help:
    -h      -- print basic options
    -h long -- print more options
    -h full -- print all options (including all format and codec specific options, very long)
    -h type=name -- print all options for the named decoder/encoder/demuxer/muxer/filter/bsf/protocol
    See man ffmpeg for detailed description of the options.

Per-stream options can be followed by :<stream_spec> to apply that option to specific streams only. <stream_spec> can be a stream index, or v/a/s for video/audio/subtitle (see manual for full syntax).

Print help / information / capabilities:
-L                  show license
-h <topic>          show help
-version            show version
-muxers             show available muxers
-demuxers           show available demuxers
-devices            show available devices
-decoders           show available decoders
-encoders           show available encoders
-filters            show available filters
-pix_fmts           show available pixel formats
-layouts            show standard channel layouts
-sample_fmts        show available audio sample formats
*/

fn parse_args(args: &mut Vec<String>, mode: &mut SSTVMode, volume: &mut f32, sample_rate: &mut u32, infile_path: &mut String, outfile_path: &mut String) {
    let helpmsg = format!(r#"Usage: {} infile [options]
Options:
  -h, --help                Display this text
  --version                 Display version information
  -m, --mode <mode>         Specify SSTV mode(default Scottie S1)
  -v, --volume <num>        Specify audio volume percentage(0-100, default 50)
  -s, --sample-rate <num>   Specify audio sample rate(default 44100)
  -o <filename>             Specify output file name

Modes:
   Mode name      Transfer time(s)     Resolution     Speed(lpm)
  Martin1, M1          114              320x256          134
  Martin2, M2           58              160x256          264
  Martin3, M3           57              320x128          134
  Martin4, M4           29              160x128          264
  Scottie1, S1         110              320x256          140
  Scottie2, S2          71              160x256          216
  Scottie3, S3          55              320x128          140
  Scottie4, S4          36              160x128          216
  ScottieDX, SDX       269              320x256           57
"#, args[0]);
    let versionmsg = format!(r#"
{} Version 1.0.0 20260126
"#, args[0]);
    
    if args.len() < 2 { //at least 1 arg needed
        print!("{}", helpmsg);
        std::process::exit(0);
    }

    let mut flag_mode = false;
    let mut flag_volume = false;
    let mut flag_samplerate = false;
    let mut flag_output = false;

    for arg in args {
        let arg: &str = arg;

        if flag_mode {
            flag_mode = false;
            *mode = arg
                .parse()
                .expect("Invalid SSTV Mode");
            continue;
        }
        if flag_volume {
            flag_volume = false;
            *volume = arg.parse::<f32>().expect("Invalid Volume") / 100.0;
            *volume = (*volume).clamp(0.0, 100.0);
            continue;
        }
        if flag_samplerate {
            flag_samplerate = false;
            *sample_rate = arg.parse::<u32>().expect("Invalid Sample Rate");
            continue;
        }
        if flag_output {
            flag_output = false;
            *outfile_path = arg.to_string();
            continue;
        }

        match arg {
            "-h" | "--help" => {
                print!("{}", helpmsg);
                std::process::exit(0);
            }
            "--version" => {
                print!("{}", versionmsg);
                std::process::exit(0);
            }
            "-m" | "--mode" => {
                flag_mode = true;
            }
            "-v" | "--volume" => {
                flag_volume = true;
            }
            "-s" | "--sample-rate" => {
                flag_samplerate = true;
            }
            "-o" => {
                flag_output = true;
            }
            _ => {
                *infile_path = arg.to_string();
            }
        }
    }

    
}







fn write_vis<W: std::io::Write + std::io::Seek>(
    writer: &mut hound::WavWriter<W>,
    osc: &mut Oscillator,
    vis_code: u8
) {
    const VIS_LEADER_MS: f32 = 300.0;
    const VIS_BREAK_MS: f32 = 10.0;
    const VIS_BIT_MS: f32 = 30.0;

    const VIS_LEADER_HZ: f32 = 1900.0;
    const VIS_BIT_1_HZ: f32 = 1100.0;
    const VIS_BIT_0_HZ: f32 = 1300.0;
    const VIS_BIT_N_HZ: f32 = 1200.0;
    //write VIS
    println!("Writing VIS header");
    emit_tone(writer, osc, VIS_LEADER_HZ, VIS_LEADER_MS);
    emit_tone(writer, osc, VIS_BIT_N_HZ, VIS_BREAK_MS);
    emit_tone(writer, osc, VIS_LEADER_HZ, VIS_LEADER_MS);

    //start bit
    emit_tone(writer, osc, VIS_BIT_N_HZ, VIS_BIT_MS);

    let mut vis_code = vis_code;
    let mut parity = false;
    for _ in 0..7 {
        let bit = vis_code & 1;
        if bit == 1{
            emit_tone(writer, osc, VIS_BIT_1_HZ, VIS_BIT_MS);
            parity = !parity;
        }
        else {
            emit_tone(writer, osc, VIS_BIT_0_HZ, VIS_BIT_MS);
        }
        vis_code >>= 1;
    }
    //parity bit
    emit_tone(writer, osc, if parity {VIS_BIT_1_HZ} else {VIS_BIT_0_HZ}, VIS_BIT_MS);
    //stop bit
    emit_tone(writer, osc, VIS_BIT_N_HZ, VIS_BIT_MS);
}


fn emit_tone<W: std::io::Write + std::io::Seek>(
    writer: &mut hound::WavWriter<W>,
    osc: &mut Oscillator,
    freq_hz: f32,
    duration_ms: f32,
) {
    if let Err(e) = _emit_tone(writer, osc, freq_hz, duration_ms) {
        eprintln!("Failed to write tone: {}", e);
    }
}

fn _emit_tone<W: std::io::Write + std::io::Seek>(
    writer: &mut hound::WavWriter<W>,
    osc: &mut Oscillator,
    freq_hz: f32,
    duration_ms: f32,
) -> Result<(), hound::Error> {
    let sr = osc.sample_rate as f32;
    let exact_samples = duration_ms * sr / 1000.0;
    let total_samples = exact_samples + osc.frac_samples;
    let samples_to_write = total_samples.floor() as usize;
    osc.frac_samples = total_samples - (samples_to_write as f32);

    let is_silence = freq_hz <= 0.0;
    let phase_inc = if is_silence { 0.0 } else { 2.0 * PI * freq_hz / sr };
    let amp_scale = i16::MAX as f32 * osc.amplitude;

    for _ in 0..samples_to_write {
        let sample_f = if is_silence { 0.0 } else { osc.phase.sin() * amp_scale };
        let s_clamped = sample_f.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;

        writer.write_sample(s_clamped)?;

        if !is_silence {
            osc.phase += phase_inc;
            if osc.phase >= 2.0 * PI {
                osc.phase -= 2.0 * PI;
            }
        }
    }

    Ok(())
}




