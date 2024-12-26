use std::{
    collections::{BTreeMap, HashMap},
    usize,
};

use rand::{seq::SliceRandom, thread_rng};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Action {
    Coop,
    Deflect,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum Strategy {
    Deflect,
    TicToc,
    Coop,
    Random,
}

impl Strategy {
    fn get_action(self: &Self, history: &Vec<Action>) -> Action {
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

type Coord = (usize, usize);

#[derive(Clone, Debug)]
pub struct Agent {
    coord: Coord,
    history: HashMap<Coord, Vec<Action>>,
    strategy: Strategy,
    score: f32,
}

impl Agent {
    fn adapt(self: &mut Self, neighbors: Vec<&Agent>) {
        let best_neighbor = neighbors
            .into_iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

        if let Some(n) = best_neighbor {
            if n.score > self.score {
                self.strategy = n.strategy;
            }
        }
    }

    fn get_action(self: &Self, agent: &Agent) -> Action {
        let empty: Vec<Action> = vec![];
        let history: &Vec<Action> = self.history.get(&agent.coord).unwrap_or(&empty);
        self.strategy.get_action(history)
    }

    fn score(self: &mut Self, agnet: &Agent, other_action: Action, score: f32) {
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

    fn random(coord: Coord, strategies: &Vec<Strategy>) -> Agent {
        let mut rng = thread_rng();
        Agent::new(coord, strategies.choose(&mut rng).unwrap().to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub strategies: BTreeMap<Strategy, usize>,
    pub max_score: BTreeMap<Strategy, f32>,
    pub coop_actions: i32,
    pub snapshot: Vec<Vec<Strategy>>,
}

pub struct Environment {
    num_row: usize,
    num_col: usize,
    grid: Vec<Agent>,
}

impl Environment {
    pub fn step(&mut self) -> Metric {
        self.for_each_cell(|curr, neighbors| {
            curr.adapt(neighbors);
        });
        let mut actions: HashMap<(Coord, Coord), Action> = HashMap::new();
        self.for_each_cell(|curr, neighbors| {
            for n in neighbors {
                actions.insert((curr.coord, n.coord), curr.get_action(n));
            }
        });
        self.for_each_cell(|curr, neighbors| {
            for n in neighbors {
                let my_action = actions[&(curr.coord, n.coord)];
                let their_action = actions[&(n.coord, curr.coord)];
                curr.score(n, their_action, Environment::score(my_action, their_action));
            }
        });

        let mut strategies: BTreeMap<Strategy, usize> = BTreeMap::new();
        let mut max_score: BTreeMap<Strategy, f32> = BTreeMap::new();
        self.grid.iter().for_each(|curr| {
            let count = strategies.get(&curr.strategy).cloned().unwrap_or(0) + 1;
            strategies.insert(curr.strategy, count);

            let score = max_score.get(&curr.strategy).cloned().unwrap_or(0.0);
            max_score.insert(curr.strategy, score.max(curr.score));
        });
        let snapshot: Vec<Vec<Strategy>> = self
            .grid
            .iter()
            .map(|a| a.strategy)
            .collect::<Vec<Strategy>>()
            .chunks(self.num_col)
            .map(|c| c.iter().cloned().collect())
            .collect();

        let coop_actions = actions.values().filter(|a| (**a == Action::Coop)).count() as i32;

        Metric {
            coop_actions,
            strategies,
            max_score,
            snapshot,
        }
    }

    pub fn new(num_row: usize, num_col: usize) -> Environment {
        let strategies = vec![Strategy::Deflect, Strategy::TicToc];
        Environment::new_with_agent_func(num_row, num_col, |c| Agent::random(c, &strategies))
    }

    pub fn new_with_agent_func<F>(num_row: usize, num_col: usize, mut agent_fn: F) -> Environment
    where
        F: FnMut(Coord) -> Agent,
    {
        let mut grid: Vec<Agent> = Vec::with_capacity(num_row * num_col);
        for i in 0..num_row {
            for j in 0..num_col {
                grid.push(agent_fn((i, j)));
            }
        }

        Environment {
            num_row,
            num_col,
            grid,
        }
    }

    fn score(a: Action, b: Action) -> f32 {
        match (a, b) {
            (Action::Coop, Action::Coop) => 3.0,
            (Action::Deflect, Action::Coop) => 4.0,
            _ => 0.0,
        }
    }

    fn for_each_cell<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Agent, Vec<&Agent>),
    {
        for x in 0..self.num_row {
            for y in 0..self.num_col {
                // Make sure all indicies are unique.
                let mut all_coords: Vec<Coord> = self
                    .neighbor_coord((x, y))
                    .into_iter()
                    .chain(vec![(x, y)])
                    .collect();
                all_coords.sort();
                let before_dedup = all_coords.len();
                all_coords.dedup();
                let after_dedpup = all_coords.len();
                assert_eq!(before_dedup, after_dedpup);
                // Make sure all index are valid.
                all_coords
                    .into_iter()
                    .map(|c| self.to_vec_index(c))
                    .for_each(|i| assert!(i < self.grid.len()));

                // SAFETY: All indexes are within grid.len and are unique.
                unsafe {
                    let ptr = self.grid.as_mut_ptr();
                    let current = ptr.add(self.to_vec_index((x, y))).as_mut().unwrap();
                    let agents: Vec<&Agent> = self
                        .neighbor_coord((x, y))
                        .iter()
                        .map(|c| ptr.add(self.to_vec_index(*c)).as_ref().unwrap())
                        .collect();
                    f(current, agents);
                }
            }
        }
    }

    fn neighbor_coord(&self, coord: Coord) -> Vec<Coord> {
        let mut result: Vec<Coord> = Vec::new();
        let upper = |bond: usize| move |s: usize| (s < bond).then_some(s);
        for dx in [-1, 0, 1] {
            for dy in [-1, 0, 1] {
                if dx != 0 || dy != 0 {
                    let x = coord.0.checked_add_signed(dx).and_then(upper(self.num_row));
                    let y = coord.1.checked_add_signed(dy).and_then(upper(self.num_col));

                    if let (Some(x), Some(y)) = (x, y) {
                        result.push((x, y));
                    }
                }
            }
        }
        result
    }

    fn to_vec_index(&self, coord: Coord) -> usize {
        self.num_col * coord.0 + coord.1
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

    #[test]
    fn test_env() {
        let mut env = Environment::new(50, 50);
        let result = env.step();
        assert!(result.coop_actions < 1400 * 8 && result.coop_actions > 1100 * 8);
    }
}
