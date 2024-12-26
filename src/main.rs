use std::{fmt::Debug, thread::sleep, time::Duration};

use coop::{Agent, Environment, Metric, Strategy};
use rand::{thread_rng, Rng};
use ratatui::{
    style::{Color, Stylize},
    text::Line,
    widgets::{Paragraph, Widget},
};

fn main() {
    let mut env = Environment::new_with_agent_func(50, 50, |c| {
        let rand: f32 = thread_rng().gen();
        let prob_coop = 0.50;
        let prob_random = 0.00;
        let prob_tic = 0.0;

        let strategy = if rand < prob_coop {
            Strategy::Coop
        } else if rand < prob_coop + prob_random {
            Strategy::Random
        } else if rand < prob_coop + prob_random + prob_tic {
            Strategy::TicToc
        } else {
            Strategy::Deflect
        };
        Agent::new(c, strategy)
    });

    color_eyre::install().unwrap();
    let mut term = ratatui::init();

    for step in 1..1000 {
        let result = env.step();
        term.draw(|frame| {
            frame.render_widget(strategy_canvas(step, result), frame.area());
        })
        .unwrap();
        sleep(Duration::from_millis(10));
    }

    ratatui::restore();
}

fn strategy_color(strategy: Strategy) -> Color {
    match strategy {
        Strategy::Deflect => Color::Red,
        Strategy::Coop => Color::Green,
        Strategy::TicToc => Color::Yellow,
        Strategy::Random => Color::Magenta,
    }
}

fn strategy_canvas(step: usize, metric: Metric) -> impl Widget {
    let status_line = Line::from(format!(
        "Step: {} Agents: {:?} Score: {:?}",
        step, metric.strategies, metric.max_score
    ));
    let mut lines: Vec<Line> = metric
        .snapshot
        .iter()
        .map(|row| Line::from_iter(row.iter().map(|s| "██".fg(strategy_color(*s)))))
        .collect();
    lines.push(status_line);
    Paragraph::new(lines)
}
