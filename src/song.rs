
mod Song {
    struct SongMetadata {
        information: String
    }
    
    struct Song {
        intense: Vec<Sample>,
        flow: Vec<Sample>,
        ascendent: Vec<Sample>,
        normal: Vec<Sample>,
        calm: Vec<Sample>,
        _metadata: SongMetadata
    }
    
    impl Song {
        fn new() -> Self {
            Self {
                _metadata: SongMetadata { information: String::from("define important metadata") },
                intense: Vec::new(),
                flow: Vec::new(),
                ascendent: Vec::new(),
                normal: Vec::new(),
                calm: Vec::new()
            }
        }
    }
    
    struct Sample {
        _path: String,
        _phase: SampleType,
        _bpm: u32,
    }
    
    enum SampleType {
        Transitional,
        Introdutory,
        Consistent,
    }
}