use cpal::traits::HostTrait;
use media_source::media_source_chapter::MediaSourceChapter;
use media_source::media_source_history_item::MediaSourceHistoryItem;
use media_source::media_source_item::MediaSourceItem;
use media_source::media_source_session_key::MediaSourceSessionKey;
use mpsc::UnboundedReceiver;
use std::cmp::max;

use crate::background::media_source::MediaSource;
use rodio_experiments::source::{SeekError, SineWave};
use rodio_experiments::{DeviceSinkBuilder, FixedSource, Player as RodioPlayer, Source};
use rodio_experiments::{cpal, Decoder, MixerDeviceSink};
use std::fs::File;
use std::io;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use rodio_experiments::cpal::{BufferSize, Device, StreamConfig};
use rodio_experiments::cpal::traits::DeviceTrait;
use rodio_experiments::speakers::{Output, SpeakersBuilder};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[derive(Debug)]
pub enum PlayerCommand {
    Update(String),
    PlayTest(),
    PlayMedia(String, Duration),
    RestoreLastSession(MediaSourceHistoryItem),
    Pause(),
    Stop(),
    Play(),
    Next(),
    Previous(),
    Toggle(),
    Rewind(),
    FastForward(),
    CancelOngoing(),
    SeekRelative(i64),
    SeekTo(Duration),
    IncreaseVolume,
    DecreaseVolume,
    SetVolume(f32),
}

#[derive(Debug)]
pub enum PlayerEvent {
    Status(MediaSourceItem, String),
    Position(String, Duration),
    Stopped,
    // ExternalTrigger(TriggerAction)
}

pub struct Player {
    media_source: Arc<dyn MediaSource>,
    // media_source_tx: UnboundedSender<MediaSourceCommand>,
    preferred_device_name: String,
    fallback_device_name: String,
    stream: Option<MixerDeviceSink>, // when removed, the samples do not play
    sink: Option<RodioPlayer>,
    item: Option<MediaSourceItem>,
    session_key: MediaSourceSessionKey,
}

impl Player {
    pub fn new(
        media_source: Arc<dyn MediaSource>,
        preferred_device_name: String,
        fallback_device_name: String,
    ) -> Player {
        Self {
            media_source,
            // media_source_tx,
            preferred_device_name,
            fallback_device_name,
            stream: None,
            sink: None,
            item: None,
            session_key: MediaSourceSessionKey::new()
        }
    }

    pub fn connect_sink(&mut self) {

        let builder_option = Self::create_device_output_builder(
            self.preferred_device_name.clone(),
            self.fallback_device_name.clone(),
        );

        if let Some(builder) = builder_option && let Ok(stream) = builder.open_stream(){
            println!("sandreas: builder.stream.buffer_size: {:?}", stream.config().buffer_size());
            self.sink = Some(RodioPlayer::connect_new(stream.mixer()));
            self.stream = Some(stream);
        }
    }

    fn previous_delay(&self) -> Duration {
        // if you are within this time of a track, it does not skip to 0 but to the previous track
        Duration::from_secs(3)
    }

    fn create_device_output_builder(
        preferred_name: String,
        fallback_name: String,
    ) -> Option<DeviceSinkBuilder> {
        let host = cpal::default_host();
        let devices = host.output_devices().unwrap();

        let device: Option<Device> = {
            let mut preferred_dev: Option<cpal::Device> = None;
            let mut fallback_dev: Option<cpal::Device> = None;
            let mut first_dev: Option<cpal::Device> = None;
            for d in devices {
                // println!("====={}", d.name().unwrap().to_string());
                if d.name().unwrap() == preferred_name {
                    preferred_dev = Some(d);
                    break;
                } else if d.name().unwrap() == fallback_name {
                    fallback_dev = Some(d);
                } else if first_dev.is_none() {
                    first_dev = Some(d)
                }
            }

            if preferred_dev.is_some() {
                preferred_dev
            } else if fallback_dev.is_some() {
                fallback_dev
            } else {
                first_dev
            }
        };



        let builder: Option<DeviceSinkBuilder> = if device.is_some() {
            let selected_device: Device = device.unwrap();
            let mut config: StreamConfig = selected_device.default_output_config().unwrap().into();
            config.buffer_size = BufferSize::Fixed(2048);

            let builder_result = DeviceSinkBuilder::from_device(selected_device).unwrap();
            /*
            let config: StreamConfig = StreamConfig {
                channels: 2,
                sample_rate: 44100,
                buffer_size: BufferSize::Fixed(2048),
            };

             */
            Some(builder_result.with_config(&config))
        } else {
            None
        };

        builder
    }

    async fn play_test(&mut self) {
        if let Some(sink) = &self.sink {
            self.item = None;
            sink.clear();
            let waves = vec![230f32, 270f32, 330f32, 270f32, 230f32];
            for w in waves {
                let source = SineWave::new(w).amplify(0.1);
                sink.append(source);
                sink.play();
                sleep(Duration::from_millis(200)).await;
                sink.stop();
                sink.clear();
            }
        }
    }
    /*
        async fn find_media_item(&mut self, id: String) {

            let cmd = MediaSourceCommand::Find(FindCommand {
                id: id.to_string(),
                callback: Box::new(|item_option| {

                }),
            });


            self.media_source_tx.send(cmd).ok();


    }

     */

    async fn play_media(&mut self, id: String) -> io::Result<()> {
        if self.load_media(id, Duration::from_secs(0)).await {
            self.toggle();
        }
        Ok(())
    }

    async fn load_media(&mut self, id: String, position: Duration) -> bool {
        let self_item = self.item.clone();

        if let Some(i) = self_item
            && id == i.id
        {
            return true;
        }

        let zero_duration = Duration::from_secs(0);

        self.item = self.media_source.find(&id).await;
        if self.item.is_none() {
            return false;
        }

        let self_item = self.item.clone();
        let item = self_item.unwrap();
        let path = Path::new(item.location.as_str());
        let file_result = File::open(path);

        if let Ok(file) = file_result {
            let decoder_result = Decoder::try_from(file);
            if let Some(sink) = &self.sink && let Ok(decoder) = decoder_result{
                sink.clear();
                sink.append(decoder);
                if position > zero_duration {
                    let _ = sink.try_seek(position);
                }
                return true;
            }
        }

        false
    }

    fn toggle(&self) {
        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play()
            } else {
                sink.pause()
            }
        }
    }

    fn play(&self) {
        if let Some(sink) = &self.sink {
            sink.play();
        }
    }

    fn pause(&self) {
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }

    fn try_seek(&self, position: Duration) -> Result<(), SeekError> {
        if self.sink.is_none() {
            return Ok(());
        }
        let sink = self.sink.as_ref().unwrap();
        sink.try_seek(position)
    }

    fn chapters(&self) -> Vec<MediaSourceChapter> {
        let self_item = self.item.clone();
        if self_item.is_none() {
            return vec![];
        }
        let current_item = self_item.unwrap();
        current_item.metadata.chapters
    }

    fn next_chapter(&self) -> Option<MediaSourceChapter> {
        if let Some(sink) = &self.sink {
            let current_pos = sink.get_pos();
            let chapters = self.chapters();
            for chapter in chapters {
                if chapter.start > current_pos {
                    return Some(chapter);
                }
            }
        }
        None
    }

    fn current_chapter(&self) -> Option<MediaSourceChapter> {
        if let Some(sink) = &self.sink {
            let current_pos = sink.get_pos();
            let chapters = self.chapters();
            if chapters.is_empty() {
                return None;
            }
            for chapter in chapters {
                if chapter.start <= current_pos && chapter.end() >= current_pos {
                    return Some(chapter);
                }
            }
        }
        None
    }

    fn previous_chapter(&self) -> Option<MediaSourceChapter> {
        if let Some(sink) = &self.sink {
            let current_pos = sink.get_pos();
            let chapters = self.chapters();
            if chapters.is_empty() {
                return None;
            }
            let mut last_chapter: Option<MediaSourceChapter> = None;
            for chapter in chapters {
                if chapter.start <= current_pos && chapter.end() >= current_pos {
                    break;
                }
                last_chapter = Some(chapter);
            }
            return last_chapter;
        }
        None
    }

    // todo:
    // next, previous, set_volume, set_speed


    /// ui_percent: 0.0-100.0 → rodio gain: 0.0-1.5 (logarithmic perception)
    pub fn set_volume_percent(&self, sink: &RodioPlayer, ui_percent: f32) {
        let max_gain = 1.5f32;

        let ui_normalized = ui_percent.clamp(0.0, 100.0) / 100.0;  // 0.0-1.0

        // Map 0-100% to -80dB → 0dB linearly
        let db = ui_normalized * 80.0 - 80.0;  // 0%=-80dB, 100%=0dB

        // Convert dB to linear gain (rodio range)
        let linear_gain = 10.0f32.powf(db / 20.0);  // -80dB=0.0, 0dB=1.0

        // Scale to rodio max (1.5)
        let final_gain = linear_gain * max_gain;

        sink.set_volume(final_gain);
    }

    /// Reverse mapping: rodio gain → UI %
    pub fn get_volume_percent(&self, sink: &RodioPlayer) -> f32 {
        let max_gain = 1.5f32;

        let current_gain = sink.volume();
        let normalized_gain = current_gain / max_gain;  // 0.0-1.0

        // dB from linear gain
        let db = 20.0 * normalized_gain.log10();

        // UI % from dB
        let ui_normalized = (db + 80.0) / 80.0;
        (ui_normalized * 100.0).clamp(0.0, 100.0)
    }

    pub fn increase_volume(&self, sink: &RodioPlayer) {
        let current = self.get_volume_percent(sink);
        self.set_volume_percent(sink, current + 1f32);
    }

    pub fn decrease_volume(&self, sink: &RodioPlayer) {
        let current = self.get_volume_percent(sink);
        self.set_volume_percent(sink, current - 1f32);
    }


    pub async fn run(
        &mut self,
        cmd_tx: Arc<UnboundedSender<PlayerCommand>>,
        mut cmd_rx: UnboundedReceiver<PlayerCommand>,
        evt_tx: UnboundedSender<PlayerEvent>,
    ) {
        let mut last_sink_update_attempt = SystemTime::now();


        let mut ongoing_option: Arc<Option<JoinHandle<_>>> = Arc::new(None);

        let mut last_history_update = Arc::new(SystemTime::now());

        let mut last_player_pos = Duration::from_secs(0);

        let mut show_sink_message = true;

        loop {
            // polling in case the audio hardware has not been successfully initialized yet
            let now = SystemTime::now();

            if self.sink.is_none() && last_sink_update_attempt + Duration::from_millis(2000) < now {
                self.connect_sink();
                last_sink_update_attempt = now;
                println!("sink not available, trying to connect");
                show_sink_message = true;
                tokio::time::sleep(Duration::from_millis(200)).await;
                continue;
            }

            if let Some(sink) = &self.sink {
                if show_sink_message {
                    println!("sink is connected");
                    show_sink_message = false;
                }
                // sink.set_volume(0);
                if self.session_key.is_expired() {
                    self.session_key = MediaSourceSessionKey::new();
                } else if !sink.is_paused(){
                    self.session_key.extend_validity();
                }


                tokio::select! {

                    // this part makes the UI crash
                    /*
                    Some(btn_cmd) = button_cmd_rx.recv() => {
                        match btn_cmd {
                            PlayerCommand::HandleButton(key,action,timestamp) => {
                                println!("===== handle button =====");
                            }
                            _ => {}
                        }
                    }

                     */

                    Some(cmd) = cmd_rx.recv() => {
                        println!("============== cmd received ==============");

                        let rewind_tx = cmd_tx.clone();
                        let fast_forward_tx = cmd_tx.clone();
                        match cmd {
                            PlayerCommand::Update(s) => {
                                let _ = self.play_media(s.clone()).await;
                                // format!("Playing {}", x)
                                // todo: implement player.is_playing / player.status

                                self.update_playing_status(&evt_tx).await;
                                /*
                                if self.sink.is_paused() {
                                    let _ = evt_tx.send(PlayerEvent::Status("paused".to_string()));
                                } else {
                                    let _ = evt_tx.send(PlayerEvent::Status("playing".to_string()));
                                }

                                 */
                            }
                            PlayerCommand::PlayTest() => {
                                self.play_test().await;
                            }
                            PlayerCommand::PlayMedia(s, position) => {
                                let _ = self.load_media(s, position).await;
                                let _ = self.play();
                                self.update_playing_status(&evt_tx).await;
                            }
                            PlayerCommand::RestoreLastSession(media_source_history_item) => {
                                let media_item_id = media_source_history_item.item.id.clone();
                                let position = media_source_history_item.position;
                                self.session_key = media_source_history_item.session_key.clone();

                                let _ = self.load_media(media_item_id, position).await;
                                self.update_playing_status(&evt_tx).await;
                            }
                            PlayerCommand::Play() => {
                                self.play();
                                self.update_playing_status(&evt_tx).await;
                            }
                            PlayerCommand::Pause() => {
                                self.pause();
                                self.update_playing_status(&evt_tx).await;
                            }
                            PlayerCommand::Stop() => {
                                let _ = evt_tx.send(PlayerEvent::Stopped);
                                break;
                            },
                            PlayerCommand::Next() => {
                                let next_chapter = self.next_chapter();
                                if next_chapter.is_some() {
                                    let new_pos = next_chapter.unwrap().start;
                                    self.try_seek(new_pos).unwrap();
                                    self.update_position(&evt_tx, new_pos).await;
                                } else {
                                    sink.skip_one()
                                }
                            }
                            PlayerCommand::Previous() => {
                                let current_pos = sink.get_pos();
                                if current_pos <= self.previous_delay() {
                                    // todo: skip to previous playlist item
                                    // return
                                }

                                if let Some(current_chapter) = self.current_chapter()
                                    && current_pos - current_chapter.start > self.previous_delay() {
                                    self.try_seek(current_chapter.start).unwrap();
                                    self.update_position(&evt_tx, current_chapter.start).await;

                                } else if let Some(previous_chapter) = self.previous_chapter() {
                                    self.try_seek(previous_chapter.start).unwrap();
                                    self.update_position(&evt_tx, previous_chapter.start).await;

                                } else {
                                    let zero = Duration::from_secs(0);
                                    self.try_seek(zero).unwrap();
                                    self.update_position(&evt_tx, zero).await;
                                }
                            }
                            PlayerCommand::SeekRelative(millis) => {
                                // let new_pos = max(sink.get_pos().as_millis() as i64 + millis, 0) as u64;
                                // let _ = self.try_seek(Duration::from_millis(new_pos));
                                self.seek_relative(sink, millis);
                            }
                            PlayerCommand::SeekTo(_) => {},
                            PlayerCommand::Toggle() => {
                                self.toggle();
                                self.update_playing_status(&evt_tx).await;
                            },
                            PlayerCommand::CancelOngoing() => {
                                let option = ongoing_option.deref();
                                if let Some(ongoing) = option {
                                    ongoing.abort();
                                    ongoing_option = Arc::new(None);
                                }
                            },
                            PlayerCommand::Rewind() => {
                                 ongoing_option = Arc::new(Some(tokio::spawn(async move {
                                    loop {
                                        println!("rewind");
                                        // self.seek_relative(sink, -15000);
                                        rewind_tx.send(PlayerCommand::SeekRelative(-15000)).unwrap();
                                        tokio::time::sleep(Duration::from_millis(800)).await;
                                    }
                                })));
                            },
                            PlayerCommand::FastForward() => {
                                 ongoing_option = Arc::new(Some(tokio::spawn(async move {
                                    loop {
                                        println!("fast-forward");
                                        fast_forward_tx.send(PlayerCommand::SeekRelative(15000)).unwrap();
                                        tokio::time::sleep(Duration::from_millis(800)).await;
                                    }
                                })));
                            },
                            PlayerCommand::IncreaseVolume => self.increase_volume(sink),
                            PlayerCommand::DecreaseVolume => self.decrease_volume(sink),
                            PlayerCommand::SetVolume(percent) => self.set_volume_percent(sink, percent),
                        }
                    }

                    _ = tokio::time::sleep(Duration::from_millis(500)) => {
                        let pos = sink.get_pos();

                        if pos != last_player_pos {
                            self.update_position(&evt_tx, pos).await;
                            last_player_pos = pos;
                        }


                        if !sink.is_paused() {
                            if let Some(last_update) = self.update_history(last_history_update.clone(), pos).await {
                                last_history_update = Arc::new(last_update);
                            }
                        }
                    }
                }



            }
        }
    }

    /*
    pub async fn run_buttons(
        &mut self,
        mut cmd_rx: UnboundedReceiver<PlayerCommand>,
    ) {
        loop {
            if let Some(sink) = &self.sink {
                tokio::select! {
                    Some(cmd) = cmd_rx.recv() => {
                        println!("============== run_buttons cmd received ==============");
                        match cmd {
                            PlayerCommand::HandleButton(ButtonKey, ButtonAction, SystemTime) => {
                                if ButtonAction == ButtonAction::Release {
                                    self.toggle()
                                }
                            },
                            _ => {}
                        }
                    }

                }
            }
        }
    }
    */
    fn seek_relative(&self, sink: &RodioPlayer, millis: i64) {
        let new_pos = max(sink.get_pos().as_millis() as i64 + millis, 0) as u64;
        let _ = self.try_seek(Duration::from_millis(new_pos));
    }

    async fn update_position(&self, evt_tx: &UnboundedSender<PlayerEvent>, pos: Duration) {
        if let Some(item) = self.item.clone() {
            let _ = evt_tx.send(PlayerEvent::Position(item.id.to_string(), pos));
        }
    }

    async fn update_history(&self, last_history_update: Arc<SystemTime>, pos: Duration) -> Option<SystemTime> {
        let item_option = self.item.clone();
        if let Some(item) = item_option {
            // todo: implement history update
            // this won't work here, the increment / change of the last update makes the whole mutable requirement
            // get rid of that by using a HistoryState?
            // self.media_source.
            // self.media_source.history_update(&item.id, "", pos).await;

            if *last_history_update < SystemTime::now() - Duration::from_secs(5) {
                let history_item = MediaSourceHistoryItem::new(item, self.session_key.clone(), pos.clone(), SystemTime::now());
                let _ = self.media_source.history_update(history_item).await;

                return Some(SystemTime::now());
            }
        }
        None
    }

    async fn update_playing_status(&self, evt_tx: &UnboundedSender<PlayerEvent>) {
        if let Some(sink) = &self.sink {
            let self_item_opt = self.item.clone();
            if self_item_opt.is_none() {
                return;
            }
            let self_item_opt = self.item.clone();
            if self_item_opt.is_none() {
                return;
            }
            let self_item = self_item_opt.unwrap();
            if sink.is_paused() {
                let _ = evt_tx.send(PlayerEvent::Status(
                    self_item.clone(),
                    "paused".to_string(),
                ));
            } else {
                let _ = evt_tx.send(PlayerEvent::Status(
                    self_item.clone(),
                    "playing".to_string(),
                ));
            }
        }
    }
}