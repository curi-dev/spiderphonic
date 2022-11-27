

mod Spiderphonic {
    struct Spiderphonic {
        audio_manager: AudioManager,
        time_line: Timeline,
        initial_checkpoint: std::time::SystemTime,
        last_checkpoint: std::time::SystemTime,
        mode: Mode,
    }
    
    impl Spiderphonic {
    
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
}

