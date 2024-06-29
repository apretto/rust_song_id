use std::io::Write;
use std::fs::File;
use hound;
use rustfft::{FftPlanner, num_complex::Complex};

const CHUNK_SIZE : usize = 16 * 1024;

#[derive(Copy, Clone)]
struct FrequencyBand {
    lower_bound: i16,
    length: i16,
    peak_at: i16,
    peak_magnitude: f32,
}

trait FrequencyWithin {
    fn frequency_within(&self, frequency: i16) -> bool;
}

impl FrequencyWithin for FrequencyBand {
    fn frequency_within(&self, frequency: i16) -> bool {
        frequency >= self.lower_bound && frequency < self.lower_bound+self.length
    }
}

impl Default for FrequencyBand {
    fn default() -> Self {
        Self {
            lower_bound: 0,
            length: 0,
            peak_at: 0,
            peak_magnitude: 0.0,
        }
    }
}

fn main() {
    let mut reader = hound::WavReader::open("audio/songs/stardust-__-FREE-DOWNLOAD-CC0(chosic.com).wav").unwrap();
    
    let song_info = reader.spec();

    let mut all_samples = reader.samples::<i16>().peekable();

    let mut song_fingerprint: Vec<(Vec<i16>, f32)> = Vec::new();
    let mut fft_planner = FftPlanner::<f32>::new();
    let mut timestamp: f32 = 0.0;

    while all_samples.peek().is_some() {
        println!("Analyzing at timestamp {}", timestamp);
        let current_chunk = all_samples.by_ref().take(CHUNK_SIZE);
        let mut audio_data = vec![Complex{ re: 0.0, im: 0.0 }; CHUNK_SIZE];
        current_chunk.enumerate().for_each(|(i, sample)| {
            audio_data[i].re = sample.unwrap() as f32;
        });

        let r2c = fft_planner.plan_fft_forward(CHUNK_SIZE);

        r2c.process(&mut audio_data);
        
        let mut slice_fingerprint: Vec<FrequencyBand> = Vec::new();
        slice_fingerprint.push(FrequencyBand{
            lower_bound: 40,
            length: 40,
            ..Default::default()
        });
        slice_fingerprint.push(FrequencyBand{
            lower_bound: 80,
            length: 40,
            ..Default::default()
        });
        slice_fingerprint.push(FrequencyBand{
            lower_bound: 120,
            length: 60,
            ..Default::default()
        });
        slice_fingerprint.push(FrequencyBand{
            lower_bound: 180,
            length: 80,
            ..Default::default()
        });
        slice_fingerprint.push(FrequencyBand{
            lower_bound: 260,
            length: 140,
            ..Default::default()
        });

        audio_data.into_iter().enumerate().for_each(|frequency_info| {
            let (frequency, magnitude) = (frequency_info.0 as i16, frequency_info.1.re.ln());
            slice_fingerprint = slice_fingerprint.iter().map(|&(mut band)| {
                if band.frequency_within(frequency) && band.peak_magnitude < magnitude {
                    band.peak_magnitude = magnitude;
                    band.peak_at = frequency;
                }

                band
            }).collect();
        });

        song_fingerprint.push((
            slice_fingerprint.iter().map(|slice| slice.peak_at).collect(),
            timestamp
        ));

        timestamp += (CHUNK_SIZE as f32) / (song_info.sample_rate as f32);
    };

    let mut outfile = File::create("temp.txt").unwrap();
    song_fingerprint.iter().for_each(|slice| {
        write!(outfile, "{} {:?}\n", slice.1, slice.0).unwrap();
    });

    println!("Channels: {} - sample rate: {} - bits per sample: {} - duration: {}", song_info.channels, song_info.sample_rate, song_info.bits_per_sample, reader.duration()/song_info.sample_rate);
}
