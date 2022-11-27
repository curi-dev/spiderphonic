//use std::io::BufReader;
use std::{fs::File, thread, time, sync::{Mutex, Arc}, fmt, collections::HashMap};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use inputbot::KeybdKey;

pub mod fake_database;
use fake_database::fake_database::{SongHttpResponse, sample_response};


// transfer to Mode module (keyboardActivity)
const CALM_AVERAGE: f32 = 0.15;
const NORMAL_AVERAGE: f32 = 0.25;
const ASCENDENT_AVERAGE: f32 = 0.50;
const FLOW_AVERAGE: f32 = 0.65;
const INTENSE_AVERAGE: f32 = 0.80;


//#[derive(Eq, Hash, PartialEq)]
enum Mode {
    Intense(f32), // insane!
    Flow(f32), // perfect state
    Ascendent(f32), // growing rythm
    Normal(f32), // nothing happens
    Calm(f32) // lazy rythm
}

struct StructuredMode {
    label: Mode,
    samples: Vec<sample_response::Sample>,
    average_target: f32,
    keyboard_activity: (),
    emotion_recognition: ()
}

impl fmt::Debug for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hi")
    }
}

impl Mode {
    fn custom_unwrap(&self) -> f32  {
        match self {
            Mode::Intense(average_target) => *average_target,
            Mode::Flow(average_target) => *average_target,
            Mode::Ascendent(average_target) => *average_target,
            Mode::Normal(average_target) => *average_target,
            Mode::Calm(average_target) => *average_target,
        }
    }

    fn get_key(mode: &Mode) -> String {
        match mode {
            Mode::Intense(_) => String::from("intense"),
            Mode::Flow(_) => String::from("flow"),
            Mode::Ascendent(_) => String::from("ascendent"),
            Mode::Normal(_) => String::from("normal"),
            Mode::Calm(_) => String::from("calm"),
        }
    }
    
    fn get_one_after(&self) -> Result<Mode, ()> {
        match self {
            Mode::Intense(_) => Result::Err(()), 
            Mode::Flow(_) => {                
                Result::Ok(Mode::Intense(INTENSE_AVERAGE))
            }, 
            Mode::Ascendent(_) => {
                Result::Ok(Mode::Flow(FLOW_AVERAGE))
            },
            Mode::Normal(_) => {
                Result::Ok(Mode::Ascendent(ASCENDENT_AVERAGE))
            },
            Mode::Calm(_) => {
                Result::Ok(Mode::Normal(NORMAL_AVERAGE))
            },
        }
    }

    fn get_one_before(&self) -> Result<Mode, ()> {
        match self {
            Mode::Intense(_) => {
                Result::Ok( Mode::Flow(FLOW_AVERAGE))
            }, 
            Mode::Flow(_) => {
                Result::Ok(Mode::Ascendent(ASCENDENT_AVERAGE))
            },
            Mode::Ascendent(_) => {
                Result::Ok(Mode::Normal(NORMAL_AVERAGE))
            },
            Mode::Normal(_) => {
                Result::Ok(Mode::Calm(CALM_AVERAGE))
            },
            Mode::Calm(_) => Result::Err(())
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

struct Spiderphonic {
    audio_manager: AudioManager,
    time_line: Timeline,
    initial_checkpoint: std::time::SystemTime,
    last_checkpoint: std::time::SystemTime,
    mode: Mode,
}

impl Spiderphonic {
    fn new(mut audio_manager: AudioManager) -> Self {
        let initial_mode = Mode::Normal(NORMAL_AVERAGE);
                
        // audio_manager.set_audio(&initial_mode);
        audio_manager.start_play(&initial_mode);
        
        let now = time::SystemTime::now();
        Self {
            initial_checkpoint: now,
            last_checkpoint: now,
            mode: initial_mode,
            time_line: Timeline::new(), // better name to keep track of calculations
            audio_manager
        }
    }

    fn elapsed_time_from_start(&self) -> u64 {
        self.initial_checkpoint.elapsed().unwrap().as_secs()
    }

    fn elapsed_time_from_last(&self) -> u64 {
        self.last_checkpoint.elapsed().unwrap().as_secs()
    }


    fn update_calculations(&mut self) {
        let elapsed_time_from_last = self.elapsed_time_from_last();
        let elapsed_time_from_start = self.elapsed_time_from_start();
        let arithmetic_average_from_last = self.time_line.calculate_arithmetic_average(elapsed_time_from_last); 
        let arithmetic_average_from_start = self.time_line.calculate_arithmetic_average(elapsed_time_from_start); 

        let average_target = self.mode.custom_unwrap();
        let min_to_switch = 10;
        if (elapsed_time_from_last >= min_to_switch) && (arithmetic_average_from_last >= average_target) { 
            let total_average_target = (average_target / 15.0) * 100.0; // 25% less than short period
            if arithmetic_average_from_start >= total_average_target {
                let result = self.mode.get_one_after();          
                if result.is_ok() {
                    println!("\nincreasing mode...");
                    self.last_checkpoint = time::SystemTime::now();        
                    self.mode = result.unwrap();
                }
            } 

            self.audio_manager.update_mixer(&self.mode);
            
        }
    }

    fn update_tracker(&mut self) {

        let now = time::SystemTime::now();

        let interaction = Interaction::Enter(now);

        self.time_line.add_interaction(interaction)
    }
}

struct AudioManager {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    splitted_song: HashMap<String, Vec<sample_response::Sample>>,
    sinks: Vec<CustomSink>
}

unsafe impl Send for AudioManager {}

impl AudioManager {
    fn new(song: SongHttpResponse) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        let mut splitted_song: HashMap<String, Vec<sample_response::Sample>> = HashMap::new();
             
        for sample in song.samples {
            let key = if sample.sequence < 1.0 {
                String::from("calm")
            } else if sample.sequence >= 1.0 {
                String::from("normal")
            } else if sample.sequence >= 2.0 {
                String::from("ascendent")
            } else if sample.sequence >= 3.0 {
                String::from("flow")
            } else {
                String::from("intense")
            };
            
            splitted_song.entry(key).and_modify(move |samples| {
                samples.push(sample)
            }).or_insert(Vec::new());
        }

        for (_value, samples) in splitted_song.iter_mut() {
            samples.sort_by(|a, b| b.sequence.total_cmp(&a.sequence));
        };
              
        Self {
            stream,
            stream_handle,
            splitted_song,
            sinks: Vec::new()
        }
    }

    fn create_sink(&self) -> Sink {
        Sink::try_new(&self.stream_handle).unwrap()
    }

    fn update_mixer(&mut self, mode: &Mode) {
        // get the samples from the active mode
        let samples = self.splitted_song.get(&Mode::get_key(mode)).unwrap(); // change sequence by mode

        // create a new sink
        let new_sink = self.create_sink();

        // get the next using the last by the binded_samples
        let last_added =  self.sinks.last();

        if let Some(active_sink) = last_added { // in case i have a sink already active
            let binded_sample_id = active_sink.binded_samples.get(0); // could not have any binded id
            if binded_sample_id.is_none() { // situation at wich there is no binded sample
                println!("there is no binded id to this sample");
                return; // keeps playing? 
            }
            // handling a situation at wich there just one binded sample, what about many binded samples?
            let next_sample = samples.iter().find(|sample| sample.id == *binded_sample_id.unwrap());
            if next_sample.is_some() {
                if !active_sink.is_modular || !next_sample.unwrap().modular {
                    active_sink.sink.pause();
                } 

                let source = Decoder::new(File::open(&next_sample.unwrap().path).unwrap()).unwrap();
                new_sink.append(source);
                new_sink.play();
                
                let mut binded_samples = Vec::new();
                binded_samples.clone_from(&next_sample.unwrap().binded_samples);
                self.sinks.push(
                    CustomSink { 
                        sink: new_sink, 
                        is_modular: next_sample.unwrap().modular, 
                        binded_samples, 
                    }
                );
            } else {
                println!("something went wrong, all binded ids should come inside the same structure");
            }
        } else {
            println!("add the first sample");
        }

        std::thread::sleep(time::Duration::from_millis(1));
        
    }

    fn start_play(&mut self, mode: &Mode) {
        // first sample of all - its guarantee that there is at least one sample in the current initial mode (default or not)
        // its gonna be avaiable for the user only if there are samples for that mode
        let sample = self.splitted_song.get(&Mode::get_key(mode)).unwrap().get(0).unwrap();
        
        let sink = self.create_sink();

        let path = &sample.path;
        println!("path: {}", path);

        let source = Decoder::new(File::open(&sample.path).unwrap()).unwrap();

        sink.append(source);
        sink.play();
        
        let mut binded_samples = Vec::new();
        binded_samples.clone_from(&sample.binded_samples);
        self.sinks.push(CustomSink { sink, binded_samples, is_modular: sample.modular });

        std::thread::sleep(time::Duration::from_millis(1));
    }

}

struct CustomSink {
    sink: Sink,
    binded_samples: Vec<usize>,
    is_modular: bool
}

fn main() {
   
    let song = SongHttpResponse::get();
    
    let observer = Arc::from(Mutex::new(Spiderphonic::new(AudioManager::new(song))));
    
    let mutex_spiderphonic_for_updating = Arc::clone(&observer);
    KeybdKey::bind(KeybdKey::EnterKey, move || {
        mutex_spiderphonic_for_updating.lock().unwrap().update_tracker(); // change for update_tracker
    });

    let mutex_spiderphonic_for_calculations = Arc::clone(&observer);
    thread::spawn(move || {       
        let mut count = 1;
        
        loop { // this is wrong
            let total_elapsed_time = mutex_spiderphonic_for_calculations.lock().unwrap().elapsed_time_from_start();
            if total_elapsed_time >= count {
                mutex_spiderphonic_for_calculations.lock().unwrap().update_calculations(); 
                count += 1;
                thread::sleep(time::Duration::from_millis(1));
            }

        }
    });

    inputbot::handle_input_events();
}


