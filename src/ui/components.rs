use std::time::Duration;

use maud::{Markup, Render, html};

use super::{model_id, model_name};
use crate::ai_tasks::{TaskError, TaskResult};
use crate::api::Model;

pub fn model_column(model: Model, run_path: &str) -> Markup {
    let target = format!("{}-result", model_id(model));
    html! {
        section class={ "model-column " (model_id(model)) } {
            h2 { (model_name(model)) }
            button
                hx-post={ (run_path) "?model=" (model_id(model)) }
                hx-include="#inputs"
                hx-target={ "#" (target) }
                hx-disabled-elt="this"
            { "Run with " (model_name(model)) }
            div id=(target) {}
        }
    }
}

pub fn result<T, R: Render>(
    model: Model,
    result: Result<TaskResult<T>, TaskError>,
    elapsed: Duration,
    display: impl Fn(T) -> R,
) -> Markup {
    match result {
        Ok(result) => html! {
            dl {
                dt { "Result" } dd { (display(result.value)) }
                dt { "Input tokens" } dd { (result.input_tokens) }
                dt { "Output tokens" } dd { (result.output_tokens) }
                dt { "Cost" } dd { (format_cost(cost(model, result.input_tokens, result.output_tokens))) }
                dt { "Request time" } dd { (elapsed.as_millis()) " ms" }
            }
        },
        Err(_) => html! {
            dl.failed {
                dt { "Error" } dd { "Request failed, see server logs" }
            }
        },
    }
}

fn cost(model: Model, input_tokens: u64, output_tokens: u64) -> f64 {
    let (input_price, output_price) = match model {
        Model::Jev => (0.042, 0.0),
        Model::Gpt => (5.0, 30.0),
    };
    (input_tokens as f64 * input_price + output_tokens as f64 * output_price) / 1_000_000.0
}

fn format_cost(cost: f64) -> String {
    let text = format!("{cost:.8}");
    let text = text.trim_end_matches('0');
    match text.strip_suffix('.') {
        Some(whole) => format!("${whole}.0"),
        None => format!("${text}"),
    }
}
