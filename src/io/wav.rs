use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::str;

use byteorder::{LittleEndian, WriteBytesExt};

#[derive(Debug)]
pub struct RIFFHeader {
    pub riff: String,
    pub file_size: u32,
    pub four_cc: String,
}

impl RIFFHeader {
    fn new(bytes: &[u8; 12]) -> Result<RIFFHeader, String> {
        let riff = match str::from_utf8(&bytes[0..4]) {
            Ok(x) => x.to_string(),
            Err(e) => return Err(e.to_string()),
        };
        let file_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let four_cc = match str::from_utf8(&bytes[8..12]) {
            Ok(a) => a.to_string(),
            Err(e) => return Err(e.to_string()),
        };
        let header = RIFFHeader {
            riff,
            file_size,
            four_cc,
        };
        if file_size > 1_000_000 {
            Err(format!(
                "maximum file size is 1MB, found {:.1}MB",
                file_size as f32 / 1_000_000.0
            ))
        } else {
            Ok(header)
        }
    }

    fn write<W: Write>(self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writer.write(self.riff.as_bytes())?;
        writer.write_u32::<LittleEndian>(self.file_size)?;
        writer.write(self.four_cc.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FMTHeader {
    pub fmt: String,
    pub header_size: u32,
    pub format: u16,
    pub nchannels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
}

impl FMTHeader {
    fn new(bytes: &[u8; 24]) -> Result<FMTHeader, String> {
        let fmt = match str::from_utf8(&bytes[0..4]) {
            Ok(x) => x.to_string(),
            Err(e) => return Err(e.to_string()),
        };
        let header_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let format = u16::from_le_bytes([bytes[8], bytes[9]]);
        let nchannels = u16::from_le_bytes([bytes[10], bytes[11]]);
        let sample_rate = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let byte_rate = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let block_align = u16::from_le_bytes([bytes[20], bytes[21]]);
        let bits_per_sample = u16::from_le_bytes([bytes[22], bytes[23]]);

        let header = FMTHeader {
            fmt,
            header_size,
            format,
            nchannels,
            sample_rate,
            byte_rate,
            block_align,
            bits_per_sample,
        };
        if bits_per_sample != 16 {
            let msg = format!("currently only 16 bit numbers are supported {:?}", header);
            Err(msg)
        } else if nchannels == 0 || sample_rate == 0 || byte_rate == 0 || bits_per_sample == 0 {
            let msg = format!("insufficent information in FMT header {:?}", header);
            Err(msg)
        } else {
            Ok(header)
        }
    }

    fn write<W: Write>(self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writer.write(self.fmt.as_bytes())?;
        writer.write_u32::<LittleEndian>(self.header_size)?;
        writer.write_u16::<LittleEndian>(self.format)?;
        writer.write_u16::<LittleEndian>(self.nchannels)?;
        writer.write_u32::<LittleEndian>(self.sample_rate)?;
        writer.write_u32::<LittleEndian>(self.byte_rate)?;
        writer.write_u16::<LittleEndian>(self.block_align)?;
        writer.write_u16::<LittleEndian>(self.bits_per_sample)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct DataHeader {
    pub data: String,
    pub size: u32,
}

impl DataHeader {
    fn new(bytes: &[u8; 8]) -> Result<DataHeader, String> {
        let data = match str::from_utf8(&bytes[0..4]) {
            Ok("smpl") => {
                return Err("wav files containing a sampler chunk are not supported".to_string())
            }
            Ok("LIST") => {
                return Err("wav files containing a LIST chunk are not supported".to_string())
            }
            Ok(x) => x.to_string(),
            Err(e) => return Err(e.to_string()),
        };
        let size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let header = DataHeader { data, size };
        Ok(header)
    }

    fn write<W: Write>(self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writer.write(self.data.as_bytes())?;
        writer.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }
}

pub struct WAV {
    pub riff_header: RIFFHeader,
    pub fmt_header: FMTHeader,
    pub data_header: DataHeader,
    pub channels: Vec<Vec<i16>>,
}

impl WAV {
    pub fn from<T: Read>(mut f: T) -> Result<WAV, Box<dyn Error>> {
        let mut buf = [0u8; 12];
        f.read(&mut buf)?;
        let riff_header = RIFFHeader::new(&buf)?;

        let mut buf = [0u8; 24];
        f.read(&mut buf)?;
        let fmt_header = FMTHeader::new(&buf)?;

        let mut buf = [0u8; 8];
        f.read(&mut buf)?;
        let data_header = DataHeader::new(&buf)?;

        // for debugging
        // TODO implement as log
        // println!("riff_header: {:?}", riff_header);
        // println!("fmt_header: {:?}", fmt_header);
        // println!("data_header: {:?}", data_header);

        let n_channels: usize = fmt_header.nchannels.into();

        // TODO we can calculate the needed capacity given the header information
        let mut channels: Vec<Vec<i16>> = vec![vec![]; n_channels];

        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        for (i, sample) in buf.chunks(2).enumerate() {
            let channel = i % n_channels;
            channels[channel].push(i16::from_le_bytes([sample[0], sample[1]]));
        }

        let expected_n_samples = data_header.size / (fmt_header.nchannels as u32 * 2);

        let wav = WAV {
            riff_header,
            fmt_header,
            data_header,
            channels,
        };

        let n_samples = wav.channels[0].len() as u32;
        if n_samples != expected_n_samples {
            let msg = format!(
                "error reading samples. expected {}, found {}",
                expected_n_samples, n_samples
            );
            Err(msg.into())
        } else {
            Ok(wav)
        }
    }

    pub fn from_file(filename: &str) -> Result<WAV, Box<dyn Error>> {
        let f = File::open(filename)?;
        WAV::from(f)
    }

    pub fn write(self, filename: &str) -> Result<(), Box<dyn Error>> {
        let f = File::create(filename)?;
        let mut writer = BufWriter::new(f);
        self.riff_header.write(&mut writer)?;
        self.fmt_header.write(&mut writer)?;
        self.data_header.write(&mut writer)?;
        let n_samples = self.channels[0].len();
        for sample in 0..n_samples {
            for channel in self.channels.iter() {
                writer.write_i16::<LittleEndian>(channel[sample])?
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod there_and_back_again {
    use super::WAV;
    use std::fs::{remove_file, File};
    use std::io::Read;

    #[test]
    fn lossless_read_write_1khz_file() {
        let wav = WAV::from_file("src/demo.wav").unwrap();
        wav.write("src/tmp.wav").unwrap();

        let mut input_file = File::open("src/demo.wav").unwrap();
        let mut input = Vec::new();
        input_file.read_to_end(&mut input).unwrap();

        let mut output_file = File::open("src/tmp.wav").unwrap();
        let mut output = Vec::new();
        output_file.read_to_end(&mut output).unwrap();
        assert_eq!(input, output);

        remove_file("src/tmp.wav").unwrap();
    }
}
