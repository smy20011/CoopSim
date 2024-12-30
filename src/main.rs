use std::time::Duration;

use coop::{Agent, Environment, Metric, Strategy};
use rand::{thread_rng, Rng};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    style::{Color, Stylize},
    text::Line,
    widgets::{Paragraph, Widget},
};

enum UiState {
    Latest,
    Detach,
}

fn main() {
    let mut env = Environment::new_with_agent_func(50, 50, 0.1, |c| {
        let rand: f32 = thread_rng().gen();
        let prob_coop = 0.00;
        let prob_random = 0.10;
        let prob_tic = 0.40;

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
    let mut buffer: Vec<Metric> = Vec::new();
    let mut ui_state = UiState::Latest;
    let mut detach_step = 0;

    loop {
        buffer.push(env.step());
        let step = match ui_state {
            UiState::Latest => buffer.len().saturating_sub(1),
            UiState::Detach => detach_step,
        };

        let _ = term.draw(|frame| {
            frame.render_widget(strategy_canvas(step, buffer[step].clone()), frame.area());
        });
        if event::poll(Duration::from_millis(10)).unwrap() {
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Left => match ui_state {
                        UiState::Latest => {
                            ui_state = UiState::Detach;
                            detach_step = buffer.len().saturating_sub(0);
                        }
                        UiState::Detach => {
                            detach_step = detach_step.saturating_sub(1);
                        }
                    },
                    KeyCode::Right => match ui_state {
                        UiState::Detach => {
                            detach_step += 1;
                            detach_step = detach_step.min(buffer.len().saturating_sub(0));
                        }
                        _ => {}
                    },
                    KeyCode::Esc => {
                        ui_state = UiState::Latest;
                    }
                    KeyCode::Home => {
                        ui_state = UiState::Detach;
                        detach_step = 0;
                    }
                    KeyCode::End => {
                        ui_state = UiState::Detach;
                        detach_step = buffer.len().saturating_sub(1);
                    }
                    _ => {}
                }
                if matches!(key.code, KeyCode::Char('q')) {
                    break;
                }
            }
        }
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
