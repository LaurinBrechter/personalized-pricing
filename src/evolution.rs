use std::{collections::HashMap, fs::File};

use crate::simulation::{simulate_revenue, ProblemSettings, SimulationEvent};
use rand::Rng;
use rand_distr::Normal;

#[derive(Clone, Debug, PartialEq)]
pub enum Selection {
    Comma,
    Plus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Adaptation {
    RechenbergRule,
    None,
}

#[derive(Clone, Debug)]
pub struct AlgorithmSettings {
    pub num_generations: i32,
    pub lambda: i32, // number of offspring
    pub mu: i32,     // population size
    pub p: i32,      // number of parents participating in recombination
    pub selection: Selection,
    pub mutation_probability: f64,
    pub mutation_strength: f64,
    pub adaptation: Adaptation,
    pub rechenberg_window: i32,
}

#[derive(Clone, Debug)]
pub struct Individual {
    pub prices: HashMap<usize, HashMap<usize, Vec<f64>>>,
    pub event_history: Vec<SimulationEvent>,
    pub fitness_score: f64,
    pub ind_id: i32,
    pub avg_regret: f64,
    pub regret: f64,
}

impl Individual {
    pub fn new(
        ind_id: i32,
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
                // TODO: change back
                for _ in 0..1 {
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
            avg_regret: 0.0,
            regret: 0.0,
            ind_id: ind_id,
        };

        let result = simulate_revenue(&ind, settings);
        ind.event_history = result.event_history;
        ind.fitness_score = result.revenue;
        ind.avg_regret = result.avg_regret;
        ind.regret = result.regret;
        ind
    }

    pub fn get_price(&self, w: usize, g: usize, t: usize) -> f64 {
        self.prices[&g][&w][t]
    }
}

fn log_population(
    writer: &mut csv::Writer<std::fs::File>,
    population: &Vec<Individual>,
    generation: i32,
    type_: &str,
    algorithm_settings: &AlgorithmSettings,
    n_evals: i32,
    run_id: i32,
) {
    for individual in population.iter() {
        writer
            .write_record(&[
                run_id.to_string(),
                generation.to_string(),
                n_evals.to_string(),
                type_.to_string(),
                individual.ind_id.to_string(),
                individual.fitness_score.to_string(),
                individual.avg_regret.to_string(),
                individual.regret.to_string(),
                algorithm_settings.lambda.to_string(),
                algorithm_settings.mu.to_string(),
                algorithm_settings.p.to_string(),
                (if algorithm_settings.selection == Selection::Comma {
                    "comma"
                } else {
                    "plus"
                })
                .to_string(),
                algorithm_settings.mutation_probability.to_string(),
                algorithm_settings.mutation_strength.to_string(),
                (if algorithm_settings.adaptation == Adaptation::RechenbergRule {
                    "rechenberg"
                } else {
                    "none"
                })
                .to_string(),
            ])
            .unwrap();
    }
}

fn mutate_solution(individual: &Individual, settings: &AlgorithmSettings) -> Individual {
    let mut new_prices = individual.prices.clone();
    let mut rng = rand::thread_rng();

    // iterate over all prices and mutate them
    for group_map in new_prices.values_mut() {
        for period_prices in group_map.values_mut() {
            for price in period_prices.iter_mut() {
                // mutate with 30% probability
                if rng.gen_bool(settings.mutation_probability) {
                    let normal = Normal::new(0.0, 1.0).unwrap();
                    let mutation = settings.mutation_strength * rng.sample(normal);
                    *price += mutation;
                    if *price < 0.0 {
                        *price = 0.0;
                    }
                }
            }
        }
    }
    Individual {
        prices: new_prices,
        event_history: vec![],
        fitness_score: 0.0,
        avg_regret: 0.0,
        regret: 0.0,
        ind_id: individual.ind_id,
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

fn intermediate_recombination(individuals: Vec<Individual>, ind_id: i32) -> Individual {
    let mut prices = HashMap::new();
    let n_parents = individuals.len() as f64;

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
                    prices.get_mut(g).unwrap().get_mut(w).unwrap()[t] += price / n_parents;
                }
            }
        }
    }
    Individual {
        prices,
        event_history: vec![],
        fitness_score: 0.0,
        ind_id,
        avg_regret: 0.0,
        regret: 0.0,
    }
}

pub fn evolve_pricing(
    run_id: i32,
    settings: &ProblemSettings,
    algorithm_settings: &AlgorithmSettings,
    mut writer: &mut csv::Writer<File>,
) -> Individual {
    // population as a vector of individuals.
    let mut population: Vec<Individual> = Vec::new();
    let mut n_evals = 0;

    let mut params = algorithm_settings.clone();

    // initialize population with random solutions

    let mut ind_id = 0 as i32;

    for _ in 0..algorithm_settings.mu {
        n_evals += 1;
        population.push(Individual::new(
            ind_id,
            settings.n_visits as usize,
            settings.n_periods as usize,
            settings.n_groups as usize,
            settings,
        ));
        ind_id += 1;
    }

    let mut best_solution = population[0].clone();
    let mut best_score = population
        .iter()
        .map(|ind| ind.fitness_score)
        .fold(f64::NEG_INFINITY, f64::max);

    let mut success_count = 0;
    let mut prev_best_score = f64::NEG_INFINITY;

    // println!("Initial best score: {}", best_score);

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
            let offspring_individual = intermediate_recombination(parents, ind_id);
            ind_id += 1;
            let mut mutated_offspring = mutate_solution(&offspring_individual, &params);

            n_evals += 1;
            let simulation_result = simulate_revenue(&mutated_offspring, settings);
            avg_score += simulation_result.revenue;

            mutated_offspring.event_history = simulation_result.event_history;
            mutated_offspring.fitness_score = simulation_result.revenue;
            mutated_offspring.avg_regret = simulation_result.avg_regret;
            mutated_offspring.regret = simulation_result.regret;

            if simulation_result.revenue > gen_best_score {
                gen_best_score = simulation_result.revenue;
                gen_best_solution = mutated_offspring.clone();
            }
            offspring.push(mutated_offspring);
        }
        // avg_score /= population.len() as f64;
        // Log generation stats to CSV
        log_population(
            &mut writer,
            &population,
            gen,
            "population",
            &params,
            n_evals,
            run_id,
        );
        log_population(
            &mut writer,
            &offspring,
            gen,
            "offspring",
            &params,
            n_evals,
            run_id,
        );

        // println!(
        //     "Generation {}: Best revenue (generation) = {}, best revenue (overall) = {}",
        //     gen, gen_best_score, best_score
        // );
        if gen_best_score > best_score {
            success_count += 1;
            best_score = gen_best_score;
            best_solution = gen_best_solution.clone();
        }
        // offspring.push(gen_best_solution.clone());
        // while offspring.len() < algorithm_settings.mu as usize {
        //     offspring.push(mutate_solution(&gen_best_solution, settings));
        // }

        // Apply Rechenberg's rule
        if algorithm_settings.adaptation == Adaptation::RechenbergRule {
            if gen_best_score > prev_best_score {}

            // Check if we should adjust mutation strength
            if gen % algorithm_settings.rechenberg_window == 0 && gen > 0 {
                let success_rate =
                    success_count as f64 / algorithm_settings.rechenberg_window as f64;

                // println!("Success rate: {}", success_rate);

                params.mutation_strength = if success_rate > 0.2 {
                    params.mutation_strength * 1.22
                } else if success_rate < 0.2 {
                    params.mutation_strength * 0.82
                } else {
                    params.mutation_strength
                };

                // Reset counter for next window
                success_count = 0;
            }
            prev_best_score = gen_best_score;
        }

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
