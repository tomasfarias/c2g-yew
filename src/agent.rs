use c2g::app::Chess2Gif;
use c2g::config::{Colors, Config, Output};
use gloo_console::debug;
use serde::{Deserialize, Serialize};
use yew_agent::{HandlerId, Public, WorkerLink};

pub struct C2GWorker {
    link: WorkerLink<Self>,
}

/// The output of running c2g should be either an error or bytes of a gif.
#[derive(Serialize, Deserialize)]
pub enum C2GOutput {
    GIFBytes(Vec<u8>),
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub struct C2GInput {
    pub chess_pgn: String,
    pub dark_color: String,
    pub light_color: String,
}

impl yew_agent::Worker for C2GWorker {
    type Input = C2GInput;
    type Message = ();
    type Output = C2GOutput;
    type Reach = Public<Self>;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) {
        // no messaging
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        let colors = match Colors::from_strs(&msg.dark_color, &msg.light_color) {
            Ok(c) => c,
            Err(e) => {
                let output = Self::Output::Error(e.to_string());
                self.link.respond(id, output);
                return;
            }
        };

        let config = Config {
            output: Output::Buffer,
            colors,
            ..Config::default()
        };

        let chess2gif = match Chess2Gif::new(msg.chess_pgn, config) {
            Ok(c2g) => c2g,
            Err(e) => {
                let output = Self::Output::Error(e.to_string());
                self.link.respond(id, output);
                return;
            }
        };

        debug!("Generating GIF");

        let gif_bytes = match chess2gif.run() {
            Ok(bytes) => bytes,
            Err(e) => {
                let output = Self::Output::Error(e.to_string());
                self.link.respond(id, output);
                return;
            }
        };

        debug!("Done");

        let output = Self::Output::GIFBytes(gif_bytes.unwrap());
        self.link.respond(id, output);
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }

    fn resource_path_is_relative() -> bool {
        true
    }
}
