//use std::io::BufReader;
use std::{fs::File, thread, time, sync::{Mutex, Arc}, fmt, collections::HashMap, rc::Rc, borrow::Borrow};

use rodio::{Decoder, OutputStream, source::Source, OutputStreamHandle, Sink, dynamic_mixer, decoder::DecoderError};
use inputbot::KeybdKey;

pub mod fake_database;

use fake_database::FakeDatabase::{self, SongHttpResponse, sample_response};


// transfer to Mode module (keyboardActivity)
const CALM_AVERAGE: f32 = 0.15;
const NORMAL_AVERAGE: f32 = 0.25;
const ASCENDENT_AVERAGE: f32 = 0.50;
const FLOW_AVERAGE: f32 = 0.65;
const INTENSE_AVERAGE: f32 = 0.80;

enum Mode {
    Intense(f32, Sink), // insane!
    Flow(f32, Sink), // perfect state
    Ascendent(f32, Sink), // growing rythm
    Normal(f32, Sink), // nothing happens
    Calm(f32, Sink) // lazy rythm
}

#[derive(Eq, Hash, PartialEq)]
enum FakeMode {
    Intense, // insane!
    Flow, // perfect state
    Ascendent, // growing rythm
    Normal, // nothing happens
    Calm // lazy rythm
}

struct StructuredMode {
    label: Mode,
    samples: Vec<sample_response::Sample>,
    average_target: f32,
    keyboard_activity: (),
    emotion_recognition: ()
}

impl StructuredMode {

    fn set_sample(&mut self, samples: Vec<sample_response::Sample>) {
        self.samples = samples;
    }
}

impl fmt::Debug for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hi")
    }
}

impl Mode {
    fn custom_unwrap(&self) -> (f32, &Sink)  {
        match self {
            Mode::Intense(average_target, sink) => (*average_target, sink),
            Mode::Flow(average_target, sink) => (*average_target, sink),
            Mode::Ascendent(average_target, sink) => (*average_target, sink),
            Mode::Normal(average_target, sink) => (*average_target, sink),
            Mode::Calm(average_target, sink) => (*average_target, sink),
        }
    }
    
    fn get_one_after(&self, new_sink: Sink) -> Result<Mode, ()> {
        match self {
            Mode::Intense(_, _) => Result::Err(()), // populate with a constant
            Mode::Flow(_, audio_track) => {
                audio_track.pause();
                //new_sink.append(Decoder::new(self.get_source()).unwrap());
                Result::Ok(Mode::Intense(INTENSE_AVERAGE, new_sink))
            }, 
            Mode::Ascendent(_, audio_track) => {
                audio_track.pause();
                //new_sink.append(Decoder::new(self.get_source()).unwrap());
                Result::Ok(Mode::Flow(FLOW_AVERAGE, new_sink))
            },
            Mode::Normal(_, audio_track) => {
                audio_track.pause();
                //new_sink.append(Decoder::new(self.get_source()).unwrap());
                print!("switching to ascendent");
                Result::Ok(Mode::Ascendent(ASCENDENT_AVERAGE, new_sink))
            },
            Mode::Calm(_t, audio_track) => {
                audio_track.pause();
                //new_sink.append(Decoder::new(self.get_source()).unwrap());
                Result::Ok(Mode::Normal(NORMAL_AVERAGE, new_sink))
            },
        }
    }

    fn get_one_before(&self, new_sink: Sink) -> Result<Mode, ()> {
        match self {
            Mode::Intense(_, audio_track) => {
                audio_track.pause();
                //new_sink.append(Decoder::new(self.get_source()).unwrap());
                Result::Ok( Mode::Flow(FLOW_AVERAGE, new_sink))
            }, 
            Mode::Flow(_, audio_track) => {
                audio_track.pause();
                //new_sink.append(Decoder::new(self.get_source()).unwrap());
                Result::Ok(Mode::Ascendent(ASCENDENT_AVERAGE, new_sink))
            },
            Mode::Ascendent(_, audio_track) => {
                audio_track.pause();
                //new_sink.append(Decoder::new(self.get_source()).unwrap());
                Result::Ok(Mode::Normal(NORMAL_AVERAGE, new_sink))
            },
            Mode::Normal(_, audio_track) => {
                audio_track.pause();
                //new_sink.append(Decoder::new(self.get_source()).unwrap());
                Result::Ok(Mode::Calm(CALM_AVERAGE, new_sink))
            },
            Mode::Calm(_, _) => Result::Err(())
        }
    }

    fn get_source(&self) -> File { // become private
        match self {
            Mode::Intense(_, _) => File::open("samples/AlexGrohl - Electric Head.mp3").unwrap(),
            Mode::Flow(_, _) => File::open("samples/AlexGrohl - The Fire Gate.mp3").unwrap(),
            Mode::Ascendent(_, _) => File::open("samples/SOURWAH - Its Going Down - Instrumental Version.mp3").unwrap(),
            Mode::Normal(_, _) => File::open("samples/Sémø - Fractured Timeline.mp3").unwrap(),
            Mode::Calm(_, _) => File::open("samples/AlexGrohl - Electric Head.mp3").unwrap(),
        }
    }
}

#[derive(Debug)]
enum Interaction {
    Enter(std::time::SystemTime),
    Space(std::time::SystemTime)
}
#[derive(Debug)]
struct Timeline {
    interactions: Vec<Interaction>,
    arithmetic_average: f32,
}

impl Timeline {
    fn new() -> Self {
        Self {
            interactions: Vec::new(),
            arithmetic_average: 0.0 
        }
    }

    fn add_interaction(&mut self, interaction: Interaction) {
        self.interactions.push(interaction);
    }

    fn calculate_arithmetic_average(&mut self, elapsed_time: u64) -> f32 {
        let ocurrences = self.interactions.len();
        
        if ocurrences > 0 {
            let new_arithmetic_average = ocurrences as f32 / elapsed_time as f32;

            println!("new_arithmetic_average: {}", new_arithmetic_average);

            self.arithmetic_average = new_arithmetic_average;

            return new_arithmetic_average;
        }

        0.0
    }
}

struct Spiderphonic<'a> {
    audio_manager: AudioManager<'a>,
    time_line: Timeline,
    initial_checkpoint: std::time::SystemTime,
    last_checkpoint: std::time::SystemTime,
    mode: Mode,
}

impl Spiderphonic<'_> {

    fn new(mut audio_manager: AudioManager) -> Self {
        let initial_mode = Mode::Normal(0.25, audio_manager.create_sink());
                
        audio_manager.set_audio(&initial_mode);
        audio_manager.play(&initial_mode);
        
        let now = time::SystemTime::now();
        Self {
            initial_checkpoint: now,
            last_checkpoint: now,
            mode: initial_mode,
            time_line: Timeline::new(), // better name to keep track of calculations
            audio_manager
        }
    }

    fn get_initial_checkpoint(&self) -> u64 {
        self.initial_checkpoint.elapsed().unwrap().as_secs()
    }

    fn elapsed_time(&self) -> u64 {
        self.last_checkpoint.elapsed().unwrap().as_secs()
    }

    fn update_calculations(&mut self) {
        let elapsed_time = self.elapsed_time();
        let arithmetic_average = self.time_line.calculate_arithmetic_average(elapsed_time); // make conditionally after 90 seconds

        let (average_target, _) = self.mode.custom_unwrap();
        let min_to_switch = 10;
        if (elapsed_time >= min_to_switch) && (arithmetic_average >= average_target) { // what about decreasing effect? 90 seconds is the right time
            let result = self.mode.get_one_after(self.audio_manager.create_sink());          
            if result.is_ok() {
                println!("\nincreasing mode...{}", arithmetic_average);
                self.last_checkpoint = time::SystemTime::now();        
                self.mode = result.unwrap();
                self.audio_manager.play(&self.mode);
            }
        }
    }

    fn update_tracker(&mut self) {

        let now = time::SystemTime::now();

        let interaction = Interaction::Enter(now);

        self.time_line.add_interaction(interaction)
    }
}

struct AudioManager<'a> {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    song: SongHttpResponse, // keep original song
    splitted_song: HashMap<FakeMode, Vec<&'a sample_response::Sample>> // Mode is only an enum with no informations
}

unsafe impl Send for AudioManager<'_> {}

impl AudioManager<'_> {
    fn new(song: SongHttpResponse) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        let mut splitted_song: HashMap<FakeMode, Vec<&sample_response::Sample>> = HashMap::new();
        let samples = Rc::from(&song);
        
        for sample in samples.borrow() {
            let key = if sample.sequence < 1.0 {
                FakeMode::Calm
            } else if sample.sequence >= 1.0 {
                FakeMode::Normal
            } else if sample.sequence >= 2.0 {
                FakeMode::Ascendent
            } else if sample.sequence >= 3.0 {
                FakeMode::Flow
            } else {
                FakeMode::Intense
            };
            
            splitted_song.entry(key).and_modify(move |samples| {
                samples.push(sample)
            }).or_insert(Vec::new());
        }

        drop(samples);

        Self {
            stream,
            stream_handle,
            song,
            splitted_song
        }
    }

    fn create_sink(&self) -> Sink {
        Sink::try_new(&self.stream_handle).unwrap()
    }

    fn set_audio(&self, mode: &Mode) {
        let (_, sink) = mode.custom_unwrap();
        sink.append(Decoder::new(mode.get_source()).unwrap());
    }

    // fn create_decoder<R>(source: File) -> Result<Decoder<R>, DecoderError> {
    //     Ok(Decoder::new(source).unwrap())
    // }

    fn play(&mut self, mode: &Mode) { // can improve this repetitive code
        match mode {
            Mode::Intense(_, sink) => {
                sink.append(Decoder::new(mode.get_source()).unwrap());
                sink.play();
            },
            Mode::Flow(_, sink) => {
                sink.append(Decoder::new(mode.get_source()).unwrap());
                sink.play();
            },
            Mode::Ascendent(_, sink) => {
                sink.append(Decoder::new(mode.get_source()).unwrap());
                sink.play();
            },
            Mode::Normal(_, sink) => {
                sink.append(Decoder::new(mode.get_source()).unwrap());
                sink.play();
            },
            Mode::Calm(_, sink) => {
                sink.append(Decoder::new(mode.get_source()).unwrap());
                sink.play();
            },
        };
              
        std::thread::sleep(time::Duration::from_millis(1));
    }
}

fn main() {
   
    let song = FakeDatabase::SongHttpResponse::get();
    
    let spiderphonic = Spiderphonic::new(AudioManager::new(song));

    let observer = Arc::new(Mutex::new(spiderphonic));
    
    let mutex_observer_for_updating = Arc::clone(&observer);
    KeybdKey::bind(KeybdKey::EnterKey, move || {
        mutex_observer_for_updating.lock().unwrap().update_tracker(); // change for update_tracker
    });

    let mutex_observer_for_calculations = Arc::clone(&observer);
    thread::spawn(move || {       
        let mut count = 1;
        
        loop {
            let total_elapsed_time = mutex_observer_for_calculations.lock().unwrap().get_initial_checkpoint();
            if total_elapsed_time >= count {
                mutex_observer_for_calculations.lock().unwrap().update_calculations(); 
                count += 1;
                thread::sleep(time::Duration::from_millis(1));
            }

        }
    });

    inputbot::handle_input_events();
}


