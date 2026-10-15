mod components;
mod rules_check;

use axum::Router;
use axum::routing::get;
use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::api::Model;

const STYLE: &str = include_str!("style.css");
const HTMX: &str = "https://unpkg.com/htmx.org@2.0.4/dist/htmx.min.js";

pub struct UseCase {
    path: &'static str,
    title: &'static str,
    description: &'static str,
}

const USE_CASES: [&UseCase; 1] = [&rules_check::USE_CASE];

pub fn router() -> Router {
    Router::new().route("/", get(home)).merge(rules_check::router())
}

fn model_id(model: Model) -> &'static str {
    match model {
        Model::Jev => "jev",
        Model::Gpt => "gpt",
    }
}

fn model_name(model: Model) -> &'static str {
    match model {
        Model::Jev => "Jev",
        Model::Gpt => "GPT 5.5",
    }
}

fn layout(page_title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (page_title) " · Jev demo" }
                style { (PreEscaped(STYLE)) }
                script src=(HTMX) {}
            }
            body {
                header {
                    a.brand href="/" { "Jev demo" }
                    span.subtitle { "Jev vs GPT 5.5" }
                }
                main { (content) }
            }
        }
    }
}

async fn home() -> Markup {
    layout("Use cases", html! {
        h1 { "Use cases" }
        div.use-case-list {
            @for use_case in USE_CASES {
                a.use-case-card href=(use_case.path) {
                    h2 { (use_case.title) }
                    p { (use_case.description) }
                }
            }
        }
    })
}

fn use_case_page(use_case: &UseCase, run_path: &str, inputs: Markup) -> Markup {
    layout(use_case.title, html! {
        a.back href="/" { "← All use cases" }
        h1 { (use_case.title) }
        p.description { (use_case.description) }
        form #inputs .inputs { (inputs) }
        div.columns {
            (components::model_column(Model::Gpt, run_path))
            (components::model_column(Model::Jev, run_path))
        }
    })
}
