use crate::config::Config;
use std::collections::HashMap;
use event_emitter_rs::EventEmitter;
use serde::Deserialize;
use crate::Emitter;

pub struct ZPS {
    config: Config,
    emitter: EventEmitter
}

impl ZPS {
    pub fn new(tree: Option<&str>) -> ZPS {
        return match tree {
            Some(t) => ZPS {
                config: Config::for_tree(t.as_ref()),
                emitter: EventEmitter::new()
            },
            None => ZPS {
                config: Config::new(),
                emitter: EventEmitter::new()
            }
        }
    }

    pub fn env(&mut self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert("tree".to_string(), self.config.tree().to_str().unwrap_or("unknown").to_string());
        self.emitter.sync_emit("info", "hey dude".to_string());
        env
    }
}

impl Emitter for ZPS {
    fn on<F, T>(&mut self, event: &str, callback: F) -> String
        where
                for<'de> T: Deserialize<'de>,
                F: Fn(T) + 'static + Sync + Send
    {
        let id = self.emitter.on_limited(event, None, callback);
        return id;
    }
}