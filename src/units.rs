use std::time::Duration;

const C0_FREQ: f32 = 16.35;
const NOTES: &'static [&'static str] = &[
    "C", "C#/Db", "D", "D#/Eb", "E", "F", "F#/Gb", "G", "G#/Ab", "A", "A#/Bb", "B", "C",
];

fn freq_to_note(freq_hz: f32) -> String {
    let c0_distance = (freq_hz / C0_FREQ).log2().max(0.0);
    let semitones_from_c = (c0_distance.fract() * 12.0).round() as usize;
    let note = NOTES[semitones_from_c];
    if semitones_from_c == 12 {
        format!("{}{}", note, c0_distance.floor() + 1.0)
    } else {
        format!("{}{}", note, c0_distance.floor())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Second,
    Hz,
    Note,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mapping {
    Linear,
    Log10,
}

pub fn normalize(value: f32, scale: &Scale) -> f32 {
    match scale.mapping {
        Mapping::Linear => (value - scale.min) / (scale.max - scale.min),
        Mapping::Log10 => {
            let min = scale.min.log10().max(0.0);
            (value.log10().max(0.0) - min) / (scale.max.log10() - min)
        }
    }
}

// assumes normalized is between 0.0 and 1.0
// iced_audio::core::normal::Normal would be a more correct type
pub fn map_normalized(normalized: f32, scale: &Scale) -> f32 {
    match scale.mapping {
        Mapping::Linear => scale.min + normalized * (scale.max - scale.min),
        Mapping::Log10 => f32::powf(
            10.0,
            normalized * (scale.max.log10() - scale.min.log10().max(0.0)),
        ),
    }
}

pub fn format_unit(f: f32, unit: &Unit) -> String {
    match unit {
        Unit::Second => format!("{:?}", Duration::from_millis((f * 1000.0) as u64)),
        Unit::Hz => f.round().to_string() + " Hz",
        Unit::Note => freq_to_note(f),
    }
}

#[derive(Clone)]
pub struct Scale {
    pub unit: Unit,
    pub min: f32,
    pub max: f32,
    pub mapping: Mapping,
}

impl Scale {
    pub fn evenly_spaced_values(&self, n: usize, start_at_zero: bool) -> Vec<f32> {
        let n_steps: f32;
        if start_at_zero {
            n_steps = (n - 1) as f32;
        } else {
            n_steps = n as f32;
        }
        match self.mapping {
            Mapping::Linear => {
                let step = (self.max - self.min) / n_steps;
                (0..n).map(|i| self.min + (i as f32 * step)).collect()
            }
            Mapping::Log10 => {
                let step = (self.max.log10() - self.min.log10().max(0.0)) / n_steps;
                (0..n)
                    .map(|i| f32::powf(10.0, self.min + (i as f32 * step)))
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod test_freq_to_note {
    use super::freq_to_note;
    // using https://pages.mtu.edu/~suits/notefreqs.html as reference

    #[test]
    fn a4() {
        assert_eq!("A4", freq_to_note(440.0))
    }

    #[test]
    fn f1() {
        assert_eq!("F1", freq_to_note(43.65))
    }

    #[test]
    fn d7_sharp() {
        assert_eq!("D#/Eb7", freq_to_note(2489.02))
    }

    #[test]
    fn aprox_a4() {
        assert_eq!("A4", freq_to_note(439.8))
    }

    #[test]
    fn aprox_f1() {
        assert_eq!("F1", freq_to_note(43.7))
    }

    #[test]
    fn aprox_d7_sharp() {
        assert_eq!("D#/Eb7", freq_to_note(2500.0))
    }

    #[test]
    fn zero_frequency() {
        assert_eq!("C0", freq_to_note(0.0))
    }
}
