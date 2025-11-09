use std::io::BufReader;
use std::{fs::File, path::PathBuf};
use symphonia::core::io::MediaSourceStream;

pub async fn _playback(
    arg: &PathBuf,
) -> Result<rodio::Decoder<BufReader<MediaSourceStream>>, rodio::decoder::DecoderError> {
    let (_stream, _stream_handle) = rodio::OutputStream::try_default().unwrap();
    //let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let file = File::open(&arg).unwrap();
    let mss = MediaSourceStream::new(Box::new(file.try_clone().unwrap()), Default::default());
    let playback = rodio::Decoder::new(BufReader::new(mss));
    //Хорошей идеей будет попытка пропагировать ошибку с целью избежания падения от глюкнутого файла
    return playback;
}
/*
pub async fn get_duration_fast(path: &PathBuf) -> Result<usize, StandardError> {
   Ok(MediaFileMetadata::new(path)?._duration.unwrap_or(0.0) as usize)

}
*/
