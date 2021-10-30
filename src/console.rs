use crate::Emitter;

pub struct UI {}

impl UI {
    pub fn bind<E: Emitter>(emitter: &mut E, color: bool) {
        emitter.on("info", move |msg: String| {
            Self::info(msg, color)
        });
    }

    fn info(msg: String, color: bool) {
        if color {
            println!("{}", msg)
        } else {
            println!("{}", msg)
        }
    }
}