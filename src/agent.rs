use std::{
    collections::{HashMap},
    usize,
};

use rand::{seq::SliceRandom, thread_rng, Rng};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Action {
    Coop,
    Deflect,
}

impl Action {
    pub(crate) fn with_noise(&self, prob: f32) -> Action {
        let mut rng = thread_rng();
        if rng.gen::<f32>() < prob {
            match *self {
                Action::Coop => Action::Deflect,
                Action::Deflect => Action::Coop,
            }
        } else {
            *self
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum Strategy {
    Deflect,
    TicToc,
    Coop,
    Random,
}

impl Strategy {
    pub fn get_action(self: &Self, history: &Vec<Action>) -> Action {
        let mut rand = thread_rng();
        match *self {
            Strategy::Deflect => Action::Deflect,
            Strategy::TicToc => history.last().unwrap_or(&Action::Coop).to_owned(),
            Strategy::Coop => Action::Coop,
            Strategy::Random => [Action::Coop, Action::Deflect]
                .choose(&mut rand)
                .unwrap()
                .to_owned(),
        }
    }
}

pub type Coord = (usize, usize);

#[derive(Clone, Debug)]
pub struct Agent {
    pub coord: Coord,
    history: HashMap<Coord, Vec<Action>>,
    pub strategy: Strategy,
    pub score: f32,
}

impl Agent {
    pub fn adapt(self: &mut Self, neighbors: Vec<&Agent>) {
        let best_neighbor = neighbors
            .into_iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

        if let Some(n) = best_neighbor {
            if n.score > self.score {
                self.strategy = n.strategy;
            }
        }
    }

    pub fn get_action(self: &Self, agent: &Agent) -> Action {
        let empty: Vec<Action> = vec![];
        let history: &Vec<Action> = self.history.get(&agent.coord).unwrap_or(&empty);
        self.strategy.get_action(history)
    }

    pub fn score(self: &mut Self, agnet: &Agent, other_action: Action, score: f32) {
        if let Some(history) = self.history.get_mut(&agnet.coord) {
            history.push(other_action);
        } else {
            self.history.insert(agnet.coord, vec![other_action]);
        }
        self.score = self.score * 1.0 + score;
    }

    pub fn new(coord: Coord, strategy: Strategy) -> Agent {
        Agent {
            coord,
            history: HashMap::new(),
            strategy,
            score: 0.0,
        }
    }

    pub fn random(coord: Coord, strategies: &Vec<Strategy>) -> Agent {
        let mut rng = thread_rng();
        Agent::new(coord, strategies.choose(&mut rng).unwrap().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy() {
        let deflect_history = vec![Action::Deflect];
        let coop_history = vec![Action::Coop];
        for history in [deflect_history, coop_history] {
            assert_eq!(
                Strategy::TicToc.get_action(&history),
                *history.last().unwrap()
            );
            assert_eq!(Strategy::Coop.get_action(&history), Action::Coop);
            assert_eq!(Strategy::Deflect.get_action(&history), Action::Deflect);
        }
    }

    #[test]
    fn test_agent() {
        let mut agent = Agent::new((0, 0), Strategy::TicToc);
        let mut other_agent = Agent::new((0, 1), Strategy::Deflect);

        assert_eq!(agent.get_action(&other_agent), Action::Coop);
        assert_eq!(other_agent.get_action(&agent), Action::Deflect);

        agent.score(&other_agent, Action::Deflect, 0.0);
        other_agent.score(&agent, Action::Coop, 3.0);

        assert_eq!(agent.get_action(&other_agent), Action::Deflect);
        assert_eq!(other_agent.get_action(&agent), Action::Deflect);

        agent.adapt(vec![&other_agent]);
        assert_eq!(agent.strategy, Strategy::Deflect);
    }
}
