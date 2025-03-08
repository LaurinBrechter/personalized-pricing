use std::collections::HashMap;

use crate::simulation::{simulate_revenue, ProblemSettings, Solution};
use rand::Rng;

pub struct AlgorithmSettings {
    pub num_generations: i32,
    pub population_size: i32,
}

fn mutate_solution(solution: &Solution, _settings: &ProblemSettings) -> Solution {
    let mut new_prices = solution.prices.clone();
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
    Solution { prices: new_prices }
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

fn intermediate_recombination(solutions: Vec<Solution>) -> Solution {
    let mut prices = HashMap::new();

    let n_groups = solutions[0].prices.len();
    let n_visits = solutions[0].prices[&0].len();
    let n_periods = solutions[0].prices[&0][&0].len();

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
    for (s) in solutions.iter() {
        for (g, group_map) in s.prices.iter() {
            for (w, period_prices) in group_map.iter() {
                for (t, price) in period_prices.iter().enumerate() {
                    prices.get_mut(g).unwrap().get_mut(w).unwrap()[t] += price;
                }
            }
        }
    }
    Solution { prices }
}

pub fn evolve_pricing(
    settings: &ProblemSettings,
    algorithm_settings: &AlgorithmSettings,
) -> Solution {
    // population as a vector of solutions.
    let mut population: Vec<Solution> = Vec::new();

    // initialize population with random solutions
    for _ in 0..algorithm_settings.population_size {
        population.push(Solution::new(
            settings.n_visits as usize,
            settings.n_periods as usize,
            settings.n_groups as usize,
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
        let mut new_population: Vec<Solution> = Vec::new();
        let mut gen_best_solution = population[0].clone();
        let mut gen_best_score = 0.0;

        // Open/create CSV file for logging

        let mut avg_score = 0.0;
        for candidate in &population {
            let score = simulate_revenue(candidate, settings);
            avg_score += score;
            if score > gen_best_score {
                gen_best_score = score;
                gen_best_solution = candidate.clone();
            }
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
        new_population.push(gen_best_solution.clone());
        while new_population.len() < algorithm_settings.population_size as usize {
            new_population.push(mutate_solution(&gen_best_solution, settings));
        }
        population = new_population;
    }
    println!("Best overall revenue found: {}", best_score);
    best_solution
}
