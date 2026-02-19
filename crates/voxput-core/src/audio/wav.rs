use crate::audio::AudioData;
use crate::errors::Result;

/// Encode `AudioData` into raw WAV bytes (16-bit signed PCM, mono).
///
/// Builds the RIFF/WAVE header manually so we can write into a `Vec<u8>`
/// without needing hound's `WavWriter` (which requires `Seek` to back-patch
/// the header, complicating in-memory use).
pub fn encode_wav(audio: &AudioData) -> Result<Vec<u8>> {
    let num_samples = audio.samples.len() as u32;
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let sample_rate = audio.sample_rate;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align: u16 = num_channels * bits_per_sample / 8;
    let data_size = num_samples * 2; // 2 bytes per 16-bit sample
    let chunk_size = 36 + data_size; // 36 = fixed header size after "RIFF" + size field

    let mut buf = Vec::with_capacity(44 + data_size as usize);

    // RIFF chunk descriptor
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&chunk_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt sub-chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // sub-chunk size (PCM = 16)
    buf.extend_from_slice(&1u16.to_le_bytes()); // audio format (PCM = 1)
    buf.extend_from_slice(&num_channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data sub-chunk header
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());

    // Sample data
    for &sample in &audio.samples {
        let int_sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        buf.extend_from_slice(&int_sample.to_le_bytes());
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_audio(samples: Vec<f32>, sample_rate: u32) -> AudioData {
        AudioData {
            samples,
            sample_rate,
            channels: 1,
        }
    }

    #[test]
    fn encode_produces_riff_magic_bytes() {
        let audio = make_audio(vec![0.0f32; 1600], 16000);
        let bytes = encode_wav(&audio).expect("encode_wav failed");
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
    }

    #[test]
    fn encode_has_correct_length() {
        let n = 3200usize;
        let audio = make_audio(vec![0.5f32; n], 16000);
        let bytes = encode_wav(&audio).expect("encode_wav failed");
        // 44-byte header + n * 2 bytes sample data
        assert_eq!(bytes.len(), 44 + n * 2);
    }

    #[test]
    fn encode_clamps_out_of_range_samples() {
        let audio = make_audio(vec![2.0, -2.0, 0.0], 16000);
        let bytes = encode_wav(&audio).expect("should not fail on out-of-range samples");
        assert_eq!(bytes.len(), 44 + 3 * 2);
    }

    #[test]
    fn encode_empty_audio_has_44_byte_header() {
        let audio = make_audio(vec![], 16000);
        let bytes = encode_wav(&audio).expect("encode_wav on empty audio failed");
        assert_eq!(bytes.len(), 44);
        assert_eq!(&bytes[0..4], b"RIFF");
    }

    #[test]
    fn encode_sample_rate_encoded_correctly() {
        let audio = make_audio(vec![0.0; 100], 8000);
        let bytes = encode_wav(&audio).unwrap();
        // Sample rate is at bytes 24-27
        let rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        assert_eq!(rate, 8000);
    }
}
