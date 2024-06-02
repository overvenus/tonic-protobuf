mod generated {
    include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
}

pub use generated::*;
