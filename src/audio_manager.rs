 
 mod audio_manager {
    struct AudioManager {
        stream: OutputStream,
        stream_handle: OutputStreamHandle,
    }
    
    unsafe impl Send for AudioManager {}
    
    impl AudioManager {
        fn new() -> Self {
            let (stream, stream_handle) = OutputStream::try_default().unwrap();
          
            Self {
                stream,
                stream_handle
            }
        }
    
        fn create_sink(&self) -> Sink {
            Sink::try_new(&self.stream_handle).unwrap()
        }
    
        fn set_audio(&self, mode: &Mode) {
            let (_, sink) = mode.custom_unwrap();
            sink.append(Decoder::new(mode.get_source()).unwrap());
        }
    
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
 }