//use std::io::BufReader;
use std::{fs::File, thread, time::{SystemTime, Duration}, sync::{Mutex, Arc}};

use rodio::{Decoder, OutputStream, source::Source, OutputStreamHandle};
use inputbot::KeybdKey;


#[derive(Debug)]
enum Mode {
    Intense(f32), // insane!
    Flow(f32), // perfect state
    Ascendent(f32), // growing rythm
    Normal(f32), // nothing happens
    Calm(f32) // lazy rythm
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

    fn get_one_after(&self) -> Result<Mode, ()> {
        match self {
            Mode::Intense(_) => Result::Err(()),
            Mode::Flow(average_target) => Result::Ok(Mode::Intense(*average_target)), // populate with a constant
            Mode::Ascendent(average_target) => Result::Ok(Mode::Flow(*average_target)),
            Mode::Normal(average_target) => Result::Ok(Mode::Ascendent(*average_target)),
            Mode::Calm(average_target) => Result::Ok(Mode::Normal(*average_target)),
        }
    }

    fn get_one_before(&self) -> Result<Mode, ()> {
        match self {
            Mode::Intense(average_target) => Result::Ok( Mode::Flow(*average_target)), // how to set it a default
            Mode::Flow(average_target) => Result::Ok(Mode::Ascendent(*average_target)),
            Mode::Ascendent(average_target) => Result::Ok(Mode::Normal(*average_target)),
            Mode::Normal(average_target) => Result::Ok(Mode::Calm(*average_target)),
            Mode::Calm(_) => Result::Err(())
        }
    }

    fn get_source(&self) -> Decoder<File> {
        match self {
            Mode::Intense(_) => Decoder::new(File::open("samples/AlexGrohl - Electric Head.mp3").unwrap()).unwrap(),
            Mode::Flow(_) => Decoder::new(File::open("samples/AlexGrohl - Electric Head.mp3").unwrap()).unwrap(),
            Mode::Ascendent(_) => Decoder::new(File::open("samples/AlexGrohl - Electric Head.mp3").unwrap()).unwrap(),
            Mode::Normal(_) => Decoder::new(File::open("samples/AlexGrohl - Electric Head.mp3").unwrap()).unwrap(),
            Mode::Calm(_) => Decoder::new(File::open("samples/AlexGrohl - Electric Head.mp3").unwrap()).unwrap(),
        }
    }
}

#[derive(Debug)]
enum Interaction {
    Enter(std::time::SystemTime),
    Space(std::time::SystemTime)
}


struct Spiderphonic {
    player: Player,
    time_line: Timeline,
    first_checkpoint: std::time::SystemTime,
    mode: Mode,
    //warming_up: bool 
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

            self.arithmetic_average = new_arithmetic_average;

            return new_arithmetic_average;
        }

        0.0
    }
}

impl Spiderphonic {

    fn new(now: SystemTime, player: Player) -> Self {
        let initial_mode = Mode::Normal(0.35);
        player.play(initial_mode);

        Self {
            first_checkpoint: now,
            mode: initial_mode,
            time_line: Timeline::new(), // better name to keep track of calculations
            player
        }
    }

    fn elapsed_time(&self) -> u64 {
        self.first_checkpoint.elapsed().unwrap().as_secs()
    }

    fn update_calculations(&mut self) {
        let elapsed_time = self.first_checkpoint.elapsed().unwrap().as_secs();
        let arithmetic_average = self.time_line.calculate_arithmetic_average(elapsed_time); // make conditionally after 90 seconds

        if elapsed_time >= 90 && (arithmetic_average >= self.mode.custom_unwrap()) { // what about decreasing effect?
            let result = self.mode.get_one_after();          
            if result.is_ok() {
                self.mode = result.unwrap(); // here its where i could press play again
                self.player.play(result.unwrap());
            }
        }
    }

    fn update_tracker(&mut self) {

        let now = SystemTime::now();

        let interaction = Interaction::Enter(now);

        self.time_line.add_interaction(interaction)
    }

    fn reset() {
        println!("reset flow control");
    }

    // fn start_play(&self) {
    //     let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    //     let file = BufReader::new(File::open("samples/AlexGrohl - Electric Head.mp3").unwrap());
    //     let source = Decoder::new(file).unwrap();
    //     let source = Decoder::new(File::open("samples/AlexGrohl - Electric Head.mp3").unwrap()).unwrap();
    //     let source = self.mode.get_source();
    //     let result = stream_handle.play_raw(source.convert_samples());
    //     std::thread::sleep(std::time::Duration::from_secs(120));
    // }
}

struct Player {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl Player {
    fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
      
        Self {
            stream,
            stream_handle,
        }
    }

    fn play(&self, mode: Mode) {
        let result = self.stream_handle.play_raw(mode.get_source().convert_samples());

        result.expect("something went wrong!");
        
        std::thread::sleep(std::time::Duration::from_secs(120));
    }

    fn pause() {
        println!("Pause music!");
    }
}

fn main() {
    let now = SystemTime::now();
   
    let spiderphonic = Spiderphonic::new(now, Player::new());

    let observer = Arc::new(Mutex::new(spiderphonic));
    
    let mutex_observer_for_updating = Arc::clone(&observer);
    KeybdKey::bind(KeybdKey::EnterKey, move || {
        mutex_observer_for_updating.lock().unwrap().update_tracker(); // change for update_tracker
    });

    let mutex_observer_for_calculations = Arc::clone(&observer);
    thread::spawn(move || {       
        let mut count = 1;
        
        loop {
            let elapsed_time = mutex_observer_for_calculations.lock().unwrap().elapsed_time();
            if elapsed_time >= count {
                mutex_observer_for_calculations.lock().unwrap().update_calculations(); 
                count += 1;
                thread::sleep(Duration::from_millis(1));
            }

        }
    });

    inputbot::handle_input_events();
}
