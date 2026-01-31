use crate::background::database_upsert_item::DatabaseUpsertItem;
use crate::file_utils::filename_stem;
use image::imageops::FilterType;
use image::GenericImageView;
use lofty::error::LoftyError;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::picture::MimeType;
use lofty::probe::Probe;
use lofty::tag::TagType::Mp4Ilst;
use lofty::tag::{Accessor, Tag};
use media_source::media_source_chapter::MediaSourceChapter;
use media_source::media_source_image_codec::MediaSourceImageCodec;
use media_source::media_source_item::MediaSourceItem;
use media_source::media_source_metadata::MediaSourceMetadata;
use media_source::media_source_picture::MediaSourcePicture;
use mp4ameta::FreeformIdent;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use xxhash_rust::xxh3::xxh3_64;

enum MetadataRetrieverError {
    Some
}

pub struct MetadataRetriever {
    base_path: String,
    rx: tokio::sync::mpsc::Receiver<DatabaseUpsertItem>,
    tx: tokio::sync::mpsc::Sender<DatabaseUpsertItem>,
}

impl MetadataRetriever {
    pub fn new(
        base_path: String,
        rx: tokio::sync::mpsc::Receiver<DatabaseUpsertItem>,
        tx: tokio::sync::mpsc::Sender<DatabaseUpsertItem>, ) -> MetadataRetriever {
        MetadataRetriever {
            base_path,
            rx,
            tx,
        }
    }

    pub async fn retrieve_metadata(
        &mut self
    ) -> anyhow::Result<()> {
        let base_path = self.base_path.clone();
        let cache_path = format!("{}/{}", base_path.clone().trim_end_matches("/"), "cache/");

        while let Some(upsert_item) = self.rx.recv().await {

            let file_clone = upsert_item.file.clone();
            let file_path = upsert_item.file.into_os_string().into_string().unwrap();
            let meta_result = self.load_meta(file_path.clone(), cache_path.clone()).await;


            if let Ok(meta) = meta_result {
                
                let file_path = file_path.clone();
                let start_index = base_path.len();
                let rel_path = file_path[start_index..].trim_start_matches('/').to_string();
                let media_type = if rel_path.starts_with("music/") {
                    media_source::media_type::MediaType::Music
                } else if rel_path.starts_with("audiobooks/") {
                    media_source::media_type::MediaType::Audiobook
                } else {
                    media_source::media_type::MediaType::Unspecified
                };

                let title = if let Some(meta_title) = meta.clone().title {
                    meta_title
                } else if let Some(file_name_stem) = filename_stem(file_clone.as_path()) {
                    file_name_stem.to_string()
                } else {
                    String::from("")
                };

                let item = MediaSourceItem {
                    id: "".to_string(),
                    location: rel_path.clone(),
                    title,
                    media_type,
                    metadata: meta,
                };


                let send_item = DatabaseUpsertItem {
                    file: file_clone,
                    file_id: upsert_item.file_id,
                    media_source_item: Some(item),
                    model: upsert_item.model,
                };

                self.tx.send(send_item).await?;
            }

        }
        Ok(())
    }

    async fn load_meta(&self, file: String, cache_path: String) -> Result<MediaSourceMetadata, MetadataRetrieverError> {
        let result = self.extract_lofty(file, cache_path).await;
        if result.is_err() {
            return Err(MetadataRetrieverError::Some);
        }
        Ok(result.unwrap())
    }

    async fn extract_lofty(&self, path: String, cache_path: String) -> Result<MediaSourceMetadata, LoftyError> {

        let tagged_file = Probe::open(path.clone())?.guess_file_type()?.read()?;
        let tag_result = match tagged_file.primary_tag() {
            Some(primary_tag) => Some(primary_tag),
            // If the "primary" tag doesn't exist, we just grab the
            // first tag we can find. Realistically, a tag reader would likely
            // iterate through the tags to find a suitable one.
            None => tagged_file.first_tag(),
        };

        let properties = tagged_file.properties();
        let duration = properties.duration();

        if tag_result.is_none() {
            return Ok(self.empty_metadata());
        }
        let tag = tag_result.unwrap();
        let mut media_source_metadata = MediaSourceMetadata::new(
            tag.artist().map(|s| s.to_string()),
            tag.title().map(|s| s.to_string()),
            tag.album().map(|s| s.to_string()),
            None, // composer
            None, // series
            None, // part
            None, // genre
            None, // cover
            vec![], // chapters
        );
        let pictures = self.extract_pictures(cache_path.clone(), tag).await?;
        if pictures.len() > 0 {
            media_source_metadata.cover = Some(pictures[0].clone());
        }

        if tag.tag_type() == Mp4Ilst {
            self.extract_mp4_metadata(&mut media_source_metadata, path.clone(), duration);
        }
        Ok(media_source_metadata)

    }

    fn extract_mp4_metadata(&self, meta: &mut MediaSourceMetadata, path: String, duration: Duration) {
        let mut chapters: Vec<MediaSourceChapter> = Vec::new();
        let mp4tag = mp4ameta::Tag::read_from_path(path.clone()).unwrap();
        let tmp_chaps = mp4tag.chapters().iter().rev();
        let mut end = duration;
        for tmp_chap in tmp_chaps {
            let duration = end - tmp_chap.start;
            chapters.push(MediaSourceChapter::new(tmp_chap.title.clone(), tmp_chap.start, duration));
            end -= duration;
        }
        chapters.reverse();
        meta.chapters = chapters;

        let movement = mp4tag.movement();
        let movement_index = mp4tag.movement_index();
        meta.composer = mp4tag.composer().map(|s| s.to_string());

        // mp4tag.artist_sort_order()
        let series_indent = FreeformIdent::new_static("com.pilabor.tone", "SERIES");
        let series = mp4tag.strings_of(&series_indent).next();
        let part_indent = FreeformIdent::new_static("com.pilabor.tone", "PART");
        let part = mp4tag.strings_of(&part_indent).next();
        meta.genre = mp4tag.genre().map(String::from);
        // let series_part = format!("{} {}", series, part);

        if series.is_some() {
            meta.series = series.map(|s| s.to_string());
        } else if movement.is_some() {
            meta.series = movement.map(|s| s.to_string());
        }

        if part.is_some() {
            meta.part = part.map(|s| s.to_string());
        } else if movement_index.is_some() {
            meta.part = movement_index.map(|s| s.to_string());
        }
    }

    async fn extract_pictures(&self, cache_path:String, tag: &Tag) -> Result<Vec<MediaSourcePicture>, LoftyError> {
        let mut pics: Vec<MediaSourcePicture> = Vec::new();

        for pic in tag.pictures() {
            let hash_u64 = xxh3_64(&pic.data());
            let hash = format!("{:016x}", hash_u64); // 16 chars, lowercase, zero-padded

            let media_source_picture = MediaSourcePicture {
                cache_dir: cache_path.to_string(),
                hash,
                codec: self.map_encoding(pic.mime_type())
            };

            // we will always use webp for thumbnails and images
            let pic_ext = String::from("jpg");
            let pic_path_str = media_source_picture.path();
            let pic_full_path = PathBuf::from(media_source_picture.pic_full_path( pic_ext.clone()));
            let tb_full_path = PathBuf::from(media_source_picture.tb_full_path( pic_ext.clone()));
            fs::create_dir_all(pic_path_str.clone())?;

            let pic_full_path_exists = pic_full_path.exists();
            if !pic_full_path_exists {
                let _ = resize_image_bytes_to_file(&pic.data(), &pic_full_path, 368, 368);
            }

            if !tb_full_path.exists() {
                let _ = resize_image_bytes_to_file(&pic.data(), &tb_full_path, 192, 192);
            }

            pics.push(media_source_picture);
        }

        Ok(pics)
    }

    fn map_encoding(&self, p0: Option<&MimeType>) -> MediaSourceImageCodec {
        if p0.is_some() && let Some(mime_type) = p0 {
            return match mime_type {
                MimeType::Png => MediaSourceImageCodec::Png,
                MimeType::Jpeg => MediaSourceImageCodec::Jpeg,
                MimeType::Tiff => MediaSourceImageCodec::Tiff,
                MimeType::Bmp => MediaSourceImageCodec::Bmp,
                MimeType::Gif => MediaSourceImageCodec::Gif,
                _ => MediaSourceImageCodec::Unknown
            }
        }
        MediaSourceImageCodec::Unknown
    }

    pub fn empty_metadata(&self) -> MediaSourceMetadata {
        MediaSourceMetadata {
            artist: None,
            title: None,
            album: None,
            genre: None,
            composer: None,
            series: None,
            part: None,
            cover: None,
            chapters: vec![],
        }
    }

}


fn resize_image_bytes_to_file(
    image_bytes: &[u8],
    output_path: &Path,
    max_width: u32,
    max_height: u32
) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(image_bytes)?;
    let img_format =  image::ImageFormat::Jpeg;
    let (width, height) = img.dimensions();
    if width <= max_width && height <= max_height {
        img.save_with_format(output_path, img_format)?;
        return Ok(());
    }

    let aspect = width as f32 / height as f32;
    let target_width = (max_height as f32 * aspect).min(max_width as f32) as u32;
    let target_height = (max_width as f32 / aspect).min(max_height as f32) as u32;

    let resized = img.resize(target_width, target_height, FilterType::Lanczos3);
    resized.save_with_format(output_path, img_format)?;

    Ok(())
}

/*
pub async fn extract_metadata(&self, path: String, cache_path: String) -> Result<MediaSourceMetadata, MetadataExtractorError> {
        let result = self.extract_lofty(path, cache_path).await;

        if result.is_err() {
            return Err(MetadataExtractorError::Some);
        }

        Ok(result.unwrap())
    }
    async fn extract_lofty(&self, path: String, cache_path: String) -> Result<MediaSourceMetadata, LoftyError> {

        let tagged_file = Probe::open(path.clone())?.guess_file_type()?.read()?;
        let tag_result = match tagged_file.primary_tag() {
            Some(primary_tag) => Some(primary_tag),
            // If the "primary" tag doesn't exist, we just grab the
            // first tag we can find. Realistically, a tag reader would likely
            // iterate through the tags to find a suitable one.
            None => tagged_file.first_tag(),
        };

        let properties = tagged_file.properties();
        let duration = properties.duration();

        if tag_result.is_none() {
            return Ok(self.empty_metadata());
        }
        let tag = tag_result.unwrap();
        let mut media_source_metadata = MediaSourceMetadata::new(
            tag.artist().map(|s| s.to_string()),
            tag.title().map(|s| s.to_string()),
            tag.album().map(|s| s.to_string()),
            None, // composer
            None, // series
            None, // part
            None, // genre
            None, // cover
            vec![], // chapters
        );
        let pictures = self.extract_pictures(cache_path.clone(), tag).await?;
        if pictures.len() > 0 {
            media_source_metadata.cover = Some(pictures[0].clone());
        }

        if tag.tag_type() == Mp4Ilst {
            self.extract_mp4_metadata(&mut media_source_metadata, path.clone(), duration);
        }
        Ok(media_source_metadata)

    }

    fn extract_mp4_metadata(&self, meta: &mut MediaSourceMetadata, path: String, duration: Duration) {
        let mut chapters: Vec<MediaSourceChapter> = Vec::new();
        let mp4tag = mp4ameta::Tag::read_from_path(path.clone()).unwrap();
        let tmp_chaps = mp4tag.chapters().iter().rev();
        let mut end = duration;
        for tmp_chap in tmp_chaps {
            let duration = end - tmp_chap.start;
            chapters.push(MediaSourceChapter::new(tmp_chap.title.clone(), tmp_chap.start, duration));
            end -= duration;
        }
        chapters.reverse();
        meta.chapters = chapters;

        let movement = mp4tag.movement();
        let movement_index = mp4tag.movement_index();
        meta.composer = mp4tag.composer().map(|s| s.to_string());

        // mp4tag.artist_sort_order()
        let series_indent = FreeformIdent::new_static("com.pilabor.tone", "SERIES");
        let series = mp4tag.strings_of(&series_indent).next();
        let part_indent = FreeformIdent::new_static("com.pilabor.tone", "PART");
        let part = mp4tag.strings_of(&part_indent).next();
        meta.genre = mp4tag.genre().map(String::from);
        // let series_part = format!("{} {}", series, part);

        if series.is_some() {
            meta.series = series.map(|s| s.to_string());
        } else if movement.is_some() {
            meta.series = movement.map(|s| s.to_string());
        }

        if part.is_some() {
            meta.part = part.map(|s| s.to_string());
        } else if movement_index.is_some() {
            meta.part = movement_index.map(|s| s.to_string());
        }
    }

    async fn extract_pictures(&self, cache_path:String, tag: &Tag) -> Result<Vec<MediaSourcePicture>, LoftyError> {
        let mut pics: Vec<MediaSourcePicture> = Vec::new();

        for pic in tag.pictures() {
            let hash_u64 = xxh3_64(&pic.data());
            let hash = format!("{:016x}", hash_u64); // 16 chars, lowercase, zero-padded

            let media_source_picture = MediaSourcePicture {
                cache_dir: cache_path.to_string(),
                hash,
                codec: self.map_encoding(pic.mime_type())
            };

            // we will always use webp for thumbnails and images
            let pic_ext = String::from("jpg");
            let pic_path_str = media_source_picture.path();
            let pic_full_path = PathBuf::from(media_source_picture.pic_full_path( pic_ext.clone()));
            let tb_full_path = PathBuf::from(media_source_picture.tb_full_path( pic_ext.clone()));
            fs::create_dir_all(pic_path_str.clone())?;

            let pic_full_path_exists = pic_full_path.exists();
            if !pic_full_path_exists {
                let _ = resize_image_bytes_to_file(&pic.data(), &pic_full_path, 368, 368);
            }

            if !tb_full_path.exists() {
                let _ = resize_image_bytes_to_file(&pic.data(), &tb_full_path, 192, 192);
            }

            pics.push(media_source_picture);
        }

        Ok(pics)
    }


    fn map_encoding(&self, p0: Option<&MimeType>) -> MediaSourceImageCodec {
        if p0.is_some() && let Some(mime_type) = p0 {
            return match mime_type {
                MimeType::Png => MediaSourceImageCodec::Png,
                MimeType::Jpeg => MediaSourceImageCodec::Jpeg,
                MimeType::Tiff => MediaSourceImageCodec::Tiff,
                MimeType::Bmp => MediaSourceImageCodec::Bmp,
                MimeType::Gif => MediaSourceImageCodec::Gif,
                _ => MediaSourceImageCodec::Unknown
            }
        }
        MediaSourceImageCodec::Unknown
    }

    pub fn empty_metadata(&self) -> MediaSourceMetadata {
        MediaSourceMetadata {
            artist: None,
            title: None,
            album: None,
            genre: None,
            composer: None,
            series: None,
            part: None,
            cover: None,
            chapters: vec![],
        }
    }
}



 */