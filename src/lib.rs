pub mod agent;

use std::fmt;
use std::str::FromStr;

use base64::{engine::general_purpose, Engine as _};
use gloo_console::debug;
use web_sys::{HtmlInputElement, HtmlTextAreaElement, HtmlImageElement};
use yew::events::InputEvent;
use yew::functional::{use_state, use_state_eq, use_node_ref};
use yew::html::TargetCast;
use yew::{
    function_component, html, Callback, Html, Properties, UseStateHandle
};
use yew_agent::{use_bridge, UseBridgeHandle};
use wasm_bindgen_futures::spawn_local;

use crate::agent::{C2GWorker, C2GOutput, C2GInput};

#[derive(Debug, Clone)]
struct ParseThemeError;

impl fmt::Display for ParseThemeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid theme")
    }
}

/// An Enum of supported board themes
enum Theme {
    Blue,
    Green,
    Gruvbox,
    Nord,
}

impl FromStr for Theme {
    type Err = ParseThemeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blue" => Ok(Theme::Blue),
            "green" => Ok(Theme::Green),
            "gruvbox" => Ok(Theme::Gruvbox),
            "nord" => Ok(Theme::Nord),
            _ => Err(ParseThemeError),
        }
    }
}

impl Theme {
    fn dark_color(&self) -> &str {
        match self {
            Theme::Blue => "#4b648a",
            Theme::Green => "#769656",
            Theme::Gruvbox => "#282828",
            Theme::Nord => "#3b4252",
        }
    }

    fn light_color(&self) -> &str {
        match self {
            Theme::Blue => "#d0dff4",
            Theme::Green => "#eeeed2",
            Theme::Gruvbox => "#ebdbb2",
            Theme::Nord => "#d8dee9",
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct PgnInputProps {
    pub chess_pgn: UseStateHandle<String>,
}

#[function_component(PgnInput)]
fn pgn_input(props: &PgnInputProps) -> Html {
    let pgn_error = use_state_eq(|| String::new());
    let textarea_ref = use_node_ref();

    let on_chess_pgn_textarea_input = {
        let chess_pgn = props.chess_pgn.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();

            chess_pgn.set(input.value())
        })
    };

    let on_chess_pgn_file_input = {
        let chess_pgn = props.chess_pgn.clone();
        let textarea_ref = textarea_ref.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let web_file = match input.files() {
                Some(file_list) => file_list.item(0),
                None => {
                    pgn_error.set("Failed to upload PGN file. Please try again.".to_string());
                    return;
                }
            };

            let chess_pgn_setter = chess_pgn.setter();
            let textarea = textarea_ref
                .cast::<HtmlTextAreaElement>()
                .expect("textarea_ref not attached to textarea element");

            if let Some(f) = web_file {
                let file = gloo_file::File::from(f);
                let reader = gloo_file::futures::read_as_text(&file);

                spawn_local(async move {
                    let result = reader.await.unwrap();
                    debug!(&result);

                    textarea.set_value(&result);
                    chess_pgn_setter.set(result);
                });
            }
        })
    };

    html! {
        <>
            <form id="input-form" class="input-form">
                <fieldset>
                <legend>{ "Input a chess PGN" }</legend>

                <div id="input-grid" class="input-grid">
                    <div id="input-textarea-pgn" class="input-textarea-pgn">
                        <label for="chess-pgn-string">{ "Enter your PGN: " }</label>
                        <textarea ref={ textarea_ref } id="chess-pgn-string" name="chess-pgn-string" form="chess-pgn" oninput={ on_chess_pgn_textarea_input } placeholder={ "1. e4 e5 2. ke2..." }>
                        { *chess_pgn }
                        </textarea>
                    </div>

                    <div id="input-file-pgn" class="input-file-pgn">
                        <label for="chess-pgn-file">{ "Or upload a PGN file: " }</label>
                        <input type="file" id="chess-pgn-file" name="chess-pgn-file" accept=".pgn" oninput={ on_chess_pgn_file_input } />
                    </div>

                    <div id="input-url-pgn" class="input-url-pgn">
                        <label for="chess-pgn-url">{ "Or enter a chess.com or lichess.org URL: " }</label>
                        <input type="url" id="chess-pgn-url" name="chess-pgn-url" />
                    </div>
                </div>
                </fieldset>
            </form>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct ConfigFormProps {
    pub dark_color: UseStateHandle<String>,
    pub light_color: UseStateHandle<String>,
}

#[function_component(ConfigForm)]
fn config_form(props: &ConfigFormProps) -> Html {
    let update_dark_color = {
        let dark_color = props.dark_color.clone();

            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();

                dark_color.set(input.value())
            })
    };

    let update_light_color = {
        let light_color = props.light_color.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();

            light_color.set(input.value())
        })
    };

    let update_color_from_theme = {
        let dark_color = props.dark_color.clone();
        let light_color = props.light_color.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();

            if let Ok(theme) = Theme::from_str(&input.value()) {
                dark_color.set(theme.dark_color().to_string());
                light_color.set(theme.light_color().to_string());
            }
        })
    };

    html! {
        <form id="config-form" class="config-form">
            <fieldset>
            <legend>{ "Board colors" }</legend>

            <div id="colors-grid" class="colors-grid">
                <div id="color-pickers" class="color-pickers">
                    <label for="dark-color-picker">{ "Choose a dark color:" }</label>
                    <input type="color" id="dark-color-picker" name="color-picker" value={ props.dark_color.to_string() } oninput={ update_dark_color } />

                    <h3 id="colors-and">{ "AND" } </h3>

                    <label for="light-color-picker">{ "Choose a light color:" }</label>
                    <input type="color" id="light-color-picker" name="color-picker" value={ props.light_color.to_string() } oninput={ update_light_color } />
                </div>

                    <h3 id="colors-or" class="colors-or">{ "OR" } </h3>

                <div id="theme-picker" class="theme-picker">
                    <label for="theme-dropdown">{ "Set colors according to a theme: "}</label>
                    <select name="theme-dropdown" id="theme-dropdown" oninput={ update_color_from_theme }>
                        <option value="blue">{ "Blue" }</option>
                        <option value="green" selected=true>{ "Green" }</option>
                        <option value="gruvbox">{ "Gruvbox" }</option>
                        <option value="nord">{ "Nord" }</option>
                    </select>
                </div>
            </div>
            </fieldset>
        </form>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let chess_pgn = use_state_eq(|| String::new());
    let dark_color = use_state_eq(|| "#769656".to_string());
    let light_color = use_state_eq(|| "#eeeed2".to_string());

    html! {
        <>
        <header>
            <h1>{ "Chess 2 GIF" }</h1>
        </header>

        <main>

            <PgnInput chess_pgn={ chess_pgn.clone() } />
            <ConfigForm dark_color={ dark_color.clone() } light_color={ light_color.clone() } />

            <GifOutput chess_pgn={ chess_pgn.clone() } dark_color={ dark_color.clone() } light_color={ light_color.clone() } />
        </main>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct GIFOutputProps {
    pub chess_pgn: UseStateHandle<String>,
    pub dark_color: UseStateHandle<String>,
    pub light_color: UseStateHandle<String>,
}

#[function_component(GifOutput)]
fn gif_output(props: &GIFOutputProps) -> Html {
    let pgn_error = use_state(|| String::new());
    let img_node_ref = use_node_ref();

    let pgn_error_bridge = pgn_error.clone();
    let img_node_ref_bridge = img_node_ref.clone();

    let worker_bridge: UseBridgeHandle<C2GWorker> = use_bridge(move |response: C2GOutput| {
        match response {
            C2GOutput::Error(e) => {
                pgn_error_bridge.set(e);
            },
            C2GOutput::GIFBytes(bytes) => {
                let data_url = format!(
                    "data:image/gif;base64,{}",
                    general_purpose::STANDARD_NO_PAD.encode(&bytes)
                );

                let img = img_node_ref_bridge
                    .cast::<HtmlImageElement>()
                    .expect("img_node_ref not attached to img element");

                img.set_src(&data_url);
                img.set_alt("Rendered GIF");
                img.set_hidden(false);
            }
        }
    });

    let generate_gif_onclick = {
        let worker_bridge = worker_bridge.clone();
        let chess_pgn = props.chess_pgn.clone();
        let dark_color = props.dark_color.clone();
        let light_color = props.light_color.clone();

        Callback::from(move |_| {
            let input = C2GInput {
                chess_pgn: (*chess_pgn).to_owned(),
                dark_color: (*dark_color).to_owned(),
                light_color: (*light_color).to_owned(),
            };

            worker_bridge.send(input);
        })
    };

    html! {
        <>
        <div id="generate-button" class="generate-button">
            if !props.chess_pgn.is_empty() {
                <button id="generate-gif-button" value="Generate GIF!" onclick={ generate_gif_onclick }> { "Generate GIF!" } </button>
            } else {
                <button disabled=true id="generate-gif-button" value="Input PGN to Generate GIF" onclick={ generate_gif_onclick }> { "Input PGN to Generate GIF" } </button>
            }
        </div>
        <div id="gif-output" class="gif-output">
            if !pgn_error.is_empty() {
                <h3>{ format!("Failed to render GIF: {}", *pgn_error) }</h3>
            }

            <img class="chess-gif" ref={ img_node_ref } hidden=true alt="No GIF rendered" />
        </div>
        </>
    }
}
