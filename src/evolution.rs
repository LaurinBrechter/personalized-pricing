use std::collections::HashMap;

use crate::simulation::{simulate_revenue, ProblemSettings, SimulationEvent};
use rand::Rng;

#[derive(PartialEq)]
pub enum Selection {
    Comma,
    Plus,
}

pub struct AlgorithmSettings {
    pub num_generations: i32,
    pub lambda: i32, // number of offspring
    pub mu: i32,     // population size
    pub p: i32,      // number of parents participating in recombination
    pub selection: Selection,
}

#[derive(Clone, Debug)]
pub struct Individual {
    pub prices: HashMap<usize, HashMap<usize, Vec<f64>>>,
    pub event_history: Vec<SimulationEvent>,
    pub fitness_score: f64,
}

impl Individual {
    pub fn new(
        n_visits: usize,
        n_periods: usize,
        n_groups: usize,
        settings: &ProblemSettings,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut prices = HashMap::new();

        for g in 0..n_groups {
            let mut group_map = HashMap::new();
            for w in 0..n_visits {
                let mut period_prices = Vec::new();
                for _ in 0..n_periods {
                    period_prices.push(rng.gen_range(0.0..100.0));
                }
                group_map.insert(w, period_prices);
            }
            prices.insert(g, group_map);
        }

        let mut ind = Self {
            prices,
            event_history: vec![],
            fitness_score: 0.0,
        };

        let result = simulate_revenue(&ind, settings);
        ind.event_history = result.event_history;
        ind.fitness_score = result.revenue;
        ind
    }

    pub fn get_price(&self, w: usize, g: usize, t: usize) -> f64 {
        self.prices[&g][&w][t]
    }
}

fn mutate_solution(individual: &Individual, _settings: &ProblemSettings) -> Individual {
    let mut new_prices = individual.prices.clone();
    let mut rng = rand::thread_rng();

    // iterate over all prices and mutate them
    for group_map in new_prices.values_mut() {
        for period_prices in group_map.values_mut() {
            for price in period_prices.iter_mut() {
                // mutate with 30% probability
                if rng.gen_bool(0.3) {
                    let mutation: f64 = rng.gen_range(-5.0..5.0);
                    *price += mutation;
                    if *price < 0.0 {
                        *price = 0.0;
                    }
                    if *price > 100.0 {
                        *price = 100.0;
                    }
                }
            }
        }
    }
    Individual {
        prices: new_prices,
        event_history: vec![],
        fitness_score: 0.0,
    }
}

fn average_vectors(vectors: &Vec<Vec<f64>>) -> Vec<f64> {
    let mut new_vector = vec![0.0; vectors[0].len()];
    for vector in vectors {
        for (i, value) in vector.iter().enumerate() {
            new_vector[i] += value;
        }
    }

    for value in new_vector.iter_mut() {
        *value /= vectors.len() as f64;
    }

    new_vector
}

fn intermediate_recombination(individuals: Vec<Individual>) -> Individual {
    let mut prices = HashMap::new();

    let n_groups = individuals[0].prices.len();
    let n_visits = individuals[0].prices[&0].len();
    let n_periods = individuals[0].prices[&0][&0].len();

    // init empty prices
    for g in 0..n_groups {
        let mut group_map = HashMap::new();
        for w in 0..n_visits {
            let mut period_prices = Vec::new();
            for _ in 0..n_periods {
                period_prices.push(0.0);
            }
            group_map.insert(w, period_prices);
        }
        prices.insert(g, group_map);
    }
    for i in individuals.iter() {
        for (g, group_map) in i.prices.iter() {
            for (w, period_prices) in group_map.iter() {
                for (t, price) in period_prices.iter().enumerate() {
                    prices.get_mut(g).unwrap().get_mut(w).unwrap()[t] += price;
                }
            }
        }
    }
    Individual {
        prices,
        event_history: vec![],
        fitness_score: 0.0,
    }
}

pub fn evolve_pricing(
    settings: &ProblemSettings,
    algorithm_settings: &AlgorithmSettings,
) -> Individual {
    // population as a vector of individuals.
    let mut population: Vec<Individual> = Vec::new();

    // initialize population with random solutions
    for _ in 0..algorithm_settings.mu {
        population.push(Individual::new(
            settings.n_visits as usize,
            settings.n_periods as usize,
            settings.n_groups as usize,
            settings,
        ));
    }

    let mut best_solution = population[0].clone();
    let mut best_score = 0.0;

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("evolution_log.csv")
        .unwrap();
    let mut writer = csv::Writer::from_writer(file);

    writer
        .write_record(&["Generation", "Best Score", "Average Score"])
        .unwrap();

    // iterate over generations
    for gen in 0..algorithm_settings.num_generations {
        let mut offspring: Vec<Individual> = Vec::new();
        let mut gen_best_solution = population[0].clone();
        let mut gen_best_score = 0.0;

        let mut avg_score = 0.0;

        // generate offspring
        for _ in 0..algorithm_settings.lambda {
            let mut rng = rand::thread_rng();
            let mut parents = Vec::new();

            // mating selection
            for _ in 0..algorithm_settings.p {
                let parent_idx = rng.gen_range(0..algorithm_settings.mu);
                parents.push(population[parent_idx as usize].clone());
            }
            let offspring_individual = intermediate_recombination(parents);
            let mut mutated_offspring = mutate_solution(&offspring_individual, settings);
            let result: crate::simulation::SimulationResult =
                simulate_revenue(&mutated_offspring, settings);
            avg_score += result.revenue;

            mutated_offspring.event_history = result.event_history;
            mutated_offspring.fitness_score = result.revenue;
            if result.revenue > gen_best_score {
                gen_best_score = result.revenue;
                gen_best_solution = mutated_offspring.clone();
            }
            offspring.push(mutated_offspring);
        }
        avg_score /= population.len() as f64;
        // Log generation stats to CSV
        writer
            .write_record(&[
                gen.to_string(),
                gen_best_score.to_string(),
                avg_score.to_string(),
            ])
            .unwrap();
        writer.flush().unwrap();

        println!("Generation {}: Best revenue = {}", gen, gen_best_score);
        if gen_best_score > best_score {
            best_score = gen_best_score;
            best_solution = gen_best_solution.clone();
        }
        // offspring.push(gen_best_solution.clone());
        // while offspring.len() < algorithm_settings.mu as usize {
        //     offspring.push(mutate_solution(&gen_best_solution, settings));
        // }

        if algorithm_settings.selection == Selection::Comma {
            // Sort offspring by fitness score and take the mu best individuals
            offspring.sort_by(|a, b| b.fitness_score.partial_cmp(&a.fitness_score).unwrap());
            population = offspring
                .into_iter()
                .take(algorithm_settings.mu as usize)
                .collect();
        } else if algorithm_settings.selection == Selection::Plus {
            // Combine population and offspring, sort by fitness, and take mu best
            population.extend(offspring);
            population.sort_by(|a, b| b.fitness_score.partial_cmp(&a.fitness_score).unwrap());
            population = population
                .into_iter()
                .take(algorithm_settings.mu as usize)
                .collect();
        }
    }
    println!("Best overall revenue found: {}", best_score);
    best_solution
}
