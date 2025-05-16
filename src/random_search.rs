use crate::simulation::{simulate_revenue, ProblemSettings, SimulationResult};
use crate::mab::Algorithm;
use rand::Rng;
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Debug)]
pub struct RandomSearchIndividual<'a> {
    pub prices: HashMap<usize, HashMap<usize, Vec<f64>>>,
    pub fitness_score: f64,
    pub simulation_result: SimulationResult<'a>,
}

impl<'a> RandomSearchIndividual<'a> {
    pub fn new(
        n_groups: usize,
        n_visits: usize,
        n_periods: usize,
        settings: &'a ProblemSettings,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut prices = HashMap::new();

        for g in 0..n_groups {
            let mut group_map = HashMap::new();
            for w in 0..n_visits {
                let mut period_prices = Vec::new();
                for _ in 0..n_periods {
                    period_prices.push(rng.gen_range(0.0..settings.max_price)); // Random price between 0 and 100
                }
                group_map.insert(w, period_prices);
            }
            prices.insert(g, group_map);
        }

        let mut individual = Self {
            prices,
            fitness_score: 0.0,
            simulation_result: SimulationResult {
                regret: 0.0,
                n_sold: 0.0,
                avg_time_sold_at: 0.0,
                event_history: vec![],
                revenue: 0.0,
                avg_regret: 0.0,
                customers: vec![],
            },
        };

        let result = simulate_revenue(&mut individual, settings);
        individual.simulation_result = result;
        individual.fitness_score = individual.simulation_result.revenue; // Use revenue as fitness
        individual
    }
}

impl<'a> Algorithm for RandomSearchIndividual<'a> {
    fn get_price(&mut self, group_id: usize, visit: usize, period: usize) -> i32 {
        self.prices[&group_id][&0][0] as i32
    }

    fn update_average_reward(
        &mut self,
        _group_id: usize,
        _visit: usize,
        _period: usize,
        _reward: f64,
        _price: i32,
    ) {
        // Random search does not update prices based on rewards
    }
}

pub fn random_search<'a>(
    settings: &'a ProblemSettings,
    n_iterations: usize,
) -> RandomSearchIndividual<'a> {
    let mut best_individual = RandomSearchIndividual::new(
        settings.n_groups as usize,
        settings.n_visits as usize,
        settings.n_periods as usize,
        settings,
    );

    fs::remove_file("./results/random_search_log.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./results/random_search_log.csv")
        .unwrap();
    let mut writer = csv::Writer::from_writer(file);

    writer
        .write_record(&[
            "evaluation",
            "fitness",
        ])
        .unwrap();

    for iteration in 0..n_iterations {
        let candidate = RandomSearchIndividual::new(
            settings.n_groups as usize,
            settings.n_visits as usize,
            settings.n_periods as usize,
            settings,
        );

        writer
            .write_record(&[
                iteration.to_string(),
                candidate.fitness_score.to_string(),
            ])
            .unwrap();

        if candidate.fitness_score > best_individual.fitness_score {
            best_individual = candidate;
        }
    }
    println!("Best fitness score: {}", best_individual.fitness_score);

    best_individual
}