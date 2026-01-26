use std::env;
use std::str::FromStr;
use std::f32::consts::PI;

use image::GenericImageView;
use image::imageops::FilterType;

#[derive(Debug)]
enum SSTVMode {
    M1,
    M2,
    M3,
    M4,
}

impl SSTVMode {
    fn resolution(&self) -> (u32, u32) {
        match self {
            SSTVMode::M1 => (320, 256),
            SSTVMode::M2 => (160, 256),
            SSTVMode::M3 => (320, 128),
            SSTVMode::M4 => (160, 128),
        }
    }
    fn vis_code(&self) -> u8 {
        match self {
            SSTVMode::M1 => 0b0101100,
            SSTVMode::M2 => 0b0101000,
            SSTVMode::M3 => 0b0100100,
            SSTVMode::M4 => 0b0100000,
        }
    }
    fn color_scanline_ms(&self) -> f32 {
        match self {
            SSTVMode::M1 => 146.432,
            SSTVMode::M2 => 73.216,
            SSTVMode::M3 => 146.432,
            SSTVMode::M4 => 73.216,
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
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 4 { //3 arguments needed
        panic!("3 arguments needed(sstv mode, infile, outfile)");
    }

    //break down argv
    let sstv_mode: SSTVMode = argv[1]
        .parse()
        .expect("Invalid SSTV Mode");

    let infile_path =  &argv[2];
    let outfile_path =  &argv[3];

    println!("Mode: {:?}", sstv_mode);
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
    const SAMPLE_RATE: u32 = 44100;
    let spec = hound::WavSpec{
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };

    let mut writer = hound::WavWriter::create(outfile_path, spec)
        .expect("Failed to create wav file");

    let mut osc = Oscillator::new(SAMPLE_RATE, 1.0);

    write_vis(&mut writer, &mut osc, sstv_mode.vis_code());

    //sync and write image scanlines
    println!("Writing image scanlines");

    sstv_mode.write_scanlines(&mut writer, &mut osc, &image);



    writer.finalize().unwrap();

    println!("Done");


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




