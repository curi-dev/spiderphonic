pub mod fake_database {
    use std::fs::File;

    use self::sample_response::{Sample, Instrument};

    pub struct SongHttpResponse {
        pub name: String,
        pub id: usize,
        pub tags: Vec<Tags>,
        pub average_bpm: u16, // it is enough?
        pub samples: Vec<Sample>,
        _metadata: String
    }

    impl SongHttpResponse {
        pub fn get() -> Self {
            Self {
                name: String::from("The Alpha Song"),
                id: 0,
                tags: vec![Tags::Grooves, Tags::Eletric],
                average_bpm: 107, // ceil average
                samples: vec![
                    Sample { 
                        id: 0,
                        path: String::from("src/samples/drums/THE_KOUNT_94_percussion_loop_electric_tom_groove.wav"), 
                        bpm: 94, 
                        instrument: Instrument::Drum,
                        modular: false,
                        binded_samples: vec![1],
                        sequence: 1.0,
                        comments: String::from("") 
                    },
                    Sample { 
                        id: 1,
                        path: String::from("src/samples/drums/MALAY_115_drum_loop_upbeat.wav"), 
                        bpm: 115, 
                        instrument: Instrument::Drum, 
                        modular: false,
                        binded_samples: vec![0],
                        sequence: 1.1,
                        comments: String::from("") 
                    },
                    Sample { 
                        id: 2,
                        path: String::from("src/samples/drums/MALAY_115_drum_loop_upbeat_lofi.wav"), 
                        bpm: 115, 
                        instrument: Instrument::Drum,
                        modular: true,
                        binded_samples: vec![1],
                        sequence: 2.0,
                        comments: String::from("") 
                    },
                    Sample { 
                        id: 3,
                        path: String::from("src/samples/vocals/CLF_105_Vocal_Alana_Am.mp3"), 
                        bpm: 105, 
                        instrument: Instrument::Vocal, 
                        modular: true,
                        binded_samples: vec![2],
                        sequence: 2.1,
                        comments: String::from("") 
                    },
                ],
                _metadata: String::from("Define important metadata") // like format, decoder, etc and stuff
            }
        }

        pub fn get_all_samples(&self) -> &Vec<sample_response::Sample> {
            &self.samples
        }
    }

    pub enum Tags {
        Grooves,
        HipHop,
        African,
        Afrobeat,
        DeepHouse,
        Eletric,
        LoFi
    }

    pub mod sample_response {
        pub struct Sample {
            pub id: usize,
            pub path: String,
            pub bpm: u32, // it is enough?
            pub instrument: Instrument,
            pub modular: bool,
            pub binded_samples: Vec<usize>,
            pub sequence: f32,
            pub comments: String
        }
        pub enum Instrument {
            Drum,
            Bass,
            Guitar,
            Vocal
        }
        
        // impl Debug for SampleResponse {
        //     println!("Implement comments about the song");
        // }
        // database layer data
    }
}