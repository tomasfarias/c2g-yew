use std::error::Error;

use base64::{engine::general_purpose, Engine as _};
use c2g::app::Chess2Gif;
use c2g::config::{Config, Output};
use web_sys::HtmlTextAreaElement;
use yew::events::InputEvent;
use yew::functional::{use_state, use_state_eq};
use yew::html::TargetCast;
use yew::{function_component, html, Callback, Html, Renderer};

#[function_component(App)]
fn app() -> Html {
    let chess_pgn = use_state_eq(|| String::new());
    let generated_gif = use_state(|| String::new());

    let update_chess_pgn_oninput = {
        let chess_pgn = chess_pgn.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();

            chess_pgn.set(input.value())
        })
    };

    let generate_gif_onclick = {
        let generated_gif = generated_gif.clone();

        Callback::from(move |_| {
            if let Ok(gif) = generate_gif(chess_pgn.to_string()) {
                generated_gif.set(gif)
            }
        })
    };

    html! {
        <>
        <header>
            <h1>{ "Chess 2 GIF" }</h1>
        </header>

        <main>
        { if generated_gif.is_empty() {
            html! {
                <div id="showcase-gif">
                    <img src="https://media.giphy.com/media/26FL8OJzF74R5Kj6A/giphy.gif" alt="Example showcase GIF" />
                </div>
            }
        } else {
            html! {
                <div id="showcase-gif">
                    <img src={ generated_gif.to_string() } />
                </div>
            }
        } }
            <label for="chess-png-string">{ "Enter your PNG: " }</label>
            <textarea id="chess-png-string" name="chess-png-string" form="chess-png" oninput={ update_chess_pgn_oninput } placeholder={ "1.e4..." }></textarea><br /><br />
            <button value="Generate GIF" onclick={ generate_gif_onclick }> { "Generate GIF" } </button>
        </main>
        </>
    }
}

fn generate_gif(chess_pgn: String) -> Result<String, Box<dyn Error>> {
    let config = Config {
        output: Output::Buffer,
        ..Config::default()
    };

    let chess2gif = Chess2Gif::new(chess_pgn, config)?;
    let gif_bytes = chess2gif.run()?;
    let data_url = format!(
        "data:image/gif;base64,{}",
        general_purpose::STANDARD_NO_PAD.encode(&gif_bytes.unwrap())
    );
    Ok(data_url)
}

fn main() {
    Renderer::<App>::new().render();
}
