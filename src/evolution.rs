use rand::Rng;
use crate::simulation::{Solution, ProblemSettings, simulate_revenue};

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

pub fn evolve_pricing(
    settings: &ProblemSettings,
    algorithm_settings: &AlgorithmSettings,
) -> Solution {

    // population as a vector of solutions.
    let mut population: Vec<Solution> = Vec::new();
    
    // initialize population with random solutions
    for _ in 0..algorithm_settings.population_size {
        population.push(Solution::new(
            settings.n_visits,
            settings.n_periods,
            settings.n_groups,
        ));
    }

    let mut best_solution = population[0].clone();
    let mut best_score = 0.0;

    // iterate over generations
    for gen in 0..algorithm_settings.num_generations {
        let mut new_population: Vec<Solution> = Vec::new();
        let mut gen_best_solution = population[0].clone();
        let mut gen_best_score = 0.0;
        for candidate in &population {
            let score = simulate_revenue(candidate, settings);
            if score > gen_best_score {
                gen_best_score = score;
                gen_best_solution = candidate.clone();
            }
        }
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