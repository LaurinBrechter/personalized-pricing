use std::os::fd::IntoRawFd;
use std::{collections::HashMap, fs::File};

use crate::logging::log_population;
use crate::mab::Algorithm;
use crate::simulation::{simulate_revenue, ProblemSettings, SimulationEvent, SimulationResult};
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
pub struct ESSettings {
    pub num_generations: i32,
    pub lambda: i32, // number of offspring
    pub mu: i32,     // population size
    pub p: i32,      // number of parents participating in recombination
    pub selection: Selection,
    pub mutation_strength: f64,
    pub adaptation: Adaptation,
    pub rechenberg_window: i32,
    pub fn_evals: i32,
    pub resample: bool // resample parents before selection
}

#[derive(Clone, Debug)]
pub struct PriceMatrix(pub HashMap<usize, HashMap<usize, Vec<f64>>>);

impl PriceMatrix {
    pub fn get_price(&self, g: usize, w: usize, t: usize) -> f64 {
        // Check if the group exists
        match self.0.get(&g) {
            Some(group_map) => {
                // Check if the visit exists within the group
                match group_map.get(&w) {
                    Some(period_prices) => {
                        // Check if the time period is within bounds
                        if t < period_prices.len() {
                            period_prices[t]
                        } else {
                            eprintln!("ERROR: Index out of bounds - g={}, w={}, t={} (max t={})", 
                                      g, w, t, period_prices.len().saturating_sub(1));
                            0.0 // Return default value
                        }
                    }
                    None => {
                        eprintln!("ERROR: Missing visit key - g={}, w={}", g, w);
                        // Print available visit keys for debugging
                        eprintln!("Available visits for group {}: {:?}", g, group_map.keys().collect::<Vec<_>>());
                        0.0 // Return default value
                    }
                }
            }
            None => {
                eprintln!("ERROR: Missing group key - g={}", g);
                // Print available group keys for debugging
                eprintln!("Available groups: {:?}", self.0.keys().collect::<Vec<_>>());
                0.0 // Return default value
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Individual<'a> {
    pub prices: PriceMatrix,
    pub simulation_result: SimulationResult<'a>,
    pub fitness_score: f64,
    pub ind_id: i32,
}

impl<'a> Individual<'a> {
    pub fn new(
        ind_id: i32,
        n_visits: usize,
        n_periods: usize,
        n_groups: usize,
        settings: &'a ProblemSettings,
        n_evals:i32
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut prices = HashMap::new();

        for g in 0..n_groups {
            let mut group_map = HashMap::new();
            for w in 0..n_visits {
                let mut period_prices = Vec::new();
                for _ in 0..n_periods {
                    let sampled_price = rng.gen_range(0.0..settings.max_price);
                    period_prices.push(sampled_price);
                }
                group_map.insert(w, period_prices);
            }
            prices.insert(g, group_map);
        }

        let mut ind = Self {
            prices: PriceMatrix(prices),
            ind_id,
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

        // println!("Initial prices: {:?}", ind.prices.0);

        let result = simulate_and_average(&mut ind, settings, n_evals);
        ind.simulation_result = result.1;
        ind.fitness_score = result.0;
        ind
    }
}

impl<'a> Algorithm for Individual<'a> {
    fn get_price(&mut self, group_id: usize, visit: usize, period: usize) -> i32 {
        // Since ES maintains a price matrix with visits and periods,
        // we'll use the first visit and period for now
        // TODO: Extend the Algorithm trait to handle multiple visits/periods
        
        // println!(
        //     "get_price: group_id: {}, visit: {}, period: {}",
        //     group_id, visit, period
        // );
        let converted_period = period % 10;

        self.prices.get_price(group_id, 0, converted_period) as i32 // converted_period) as i32
        // self.prices.get_price(group_id, visit, period) as i32
    }

    fn update_average_reward(
        &mut self,
        _group_id: usize,
        _visit: usize,
        _period: usize,
        _reward: f64,
        _price: i32,
    ) {
        // ES doesn't update prices based on individual rewards
        // Instead, it uses the total fitness score for evolution
    }
}


fn mutate_solution<'a>(
    individual: &Individual<'a>,
    settings: &ESSettings,
) -> Individual<'a> {
    let mut new_prices = individual.prices.0.clone();
    let mut rng = rand::thread_rng();

    // iterate over all prices and mutate them
    for group_map in new_prices.values_mut() {
        for period_prices in group_map.values_mut() {
            let normal = Normal::new(0.0, 1.0).unwrap();
            for price in period_prices.iter_mut() {
                let mutation = settings.mutation_strength * rng.sample(normal);
                    *price += mutation;
                    if *price < 0.0 {
                        *price = 0.0;
                    }
            }
        }
    }
    Individual {
        prices: PriceMatrix(new_prices),
        fitness_score: 0.0,
        ind_id: individual.ind_id,
        simulation_result: SimulationResult {
            event_history: vec![],
            avg_regret: 0.0,
            regret: 0.0,
            customers: vec![],
            avg_time_sold_at: 0.0,
            n_sold: 0.0,
            revenue: 0.0,
        },
    }
}

fn mutate_solution_selective<'a>(
    individual: &Individual<'a>,
    settings: &ESSettings,
    n_changes: usize,
) -> Individual<'a> {
    let mut new_prices = individual.prices.0.clone();
    let mut rng = rand::thread_rng();
    
    // Get all possible indices for mutation
    let mut all_indices = Vec::new();
    for (g_idx, group_map) in new_prices.values_mut().enumerate() {
        for (w_idx, period_prices) in group_map.values_mut().enumerate() {
            for t_idx in 0..period_prices.len() {
                if (w_idx == 0 && t_idx < 10) {
                    all_indices.push((g_idx, w_idx, t_idx));
                }
            }
        }
    }
    
    // Select one random element to mutate
    if !all_indices.is_empty() {
        for _ in 0..n_changes {
            let (g_idx, w_idx, t_idx) = all_indices[rng.gen_range(0..all_indices.len())];
        
            // Get the element and mutate it
            if let Some(group_map) = new_prices.get_mut(&g_idx) {
                if let Some(period_prices) = group_map.get_mut(&w_idx) {
                    let price = &mut period_prices[t_idx];
                    let normal = Normal::new(0.0, 1.0).unwrap();
                    // let mutation = settings.mutation_strength * rng.sample(normal);
                    let mutation = rng.gen_range(0..700);   //settings.mutation_strength * rng.sample(normal);
        
                    *price = mutation as f64;

                    // Ensure price doesn't go below zero
                    if *price < 0.0 {
                        *price = 0.0;
                    }
                }
            }
        }
        
        // Optional debugging
        // println!("Mutated element at ({}, {}, {}): applied mutation {}", g_idx, w_idx, t_idx, mutation);
    }
    Individual {
        prices: PriceMatrix(new_prices),
        fitness_score: 0.0,
        ind_id: individual.ind_id,
        simulation_result: SimulationResult {
            event_history: vec![],
            avg_regret: 0.0,
            regret: 0.0,
            customers: vec![],
            avg_time_sold_at: 0.0,
            n_sold: 0.0,
            revenue: 0.0,
        },
    }
}



fn simulate_and_average<'a>(individual: &Individual<'a>, settings: &'a ProblemSettings, n_evals:i32) -> (f64, SimulationResult<'a>) {
    let mut total_revenue = 0.0;
    let mut rng = rand::thread_rng();
    let n_runs = 10;
    let mut best_simulation_result = individual.simulation_result.clone();

    for _ in 0..n_runs {
        let result = simulate_revenue(&mut individual.clone(), settings);
        total_revenue += result.revenue;

        if result.revenue > best_simulation_result.revenue {
            best_simulation_result = result;
        }
    }

    return (total_revenue / n_runs as f64, best_simulation_result);
}


fn intermediate_recombination<'a>(individuals: Vec<Individual<'a>>, ind_id: i32) -> Individual<'a> {
    let mut prices = HashMap::new();
    let n_parents = individuals.len() as f64;

    let n_groups = individuals[0].prices.0.len();
    let n_visits = individuals[0].prices.0[&0].len();
    let n_periods = individuals[0].prices.0[&0][&0].len();

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
        for (g, group_map) in i.prices.0.iter() {
            for (w, period_prices) in group_map.iter() {
                for (t, price) in period_prices.iter().enumerate() {
                    prices.get_mut(g).unwrap().get_mut(w).unwrap()[t] += price / n_parents;
                }
            }
        }
    }
    Individual {
        prices: PriceMatrix(prices),
        ind_id,
        fitness_score: 0.0,
        simulation_result: SimulationResult {
            event_history: vec![],
            avg_regret: 0.0,
            regret: 0.0,
            customers: vec![],
            avg_time_sold_at: 0.0,
            n_sold: 0.0,
            revenue: 0.0,
        },
    }
}

fn dominant_recombination<'a>(individuals: Vec<Individual<'a>>, ind_id: i32) -> Individual<'a> {
    let mut prices = HashMap::new();
    let mut rng = rand::thread_rng();

    let n_groups = individuals[0].prices.0.len();
    let n_visits = individuals[0].prices.0[&0].len();
    let n_periods = individuals[0].prices.0[&0][&0].len();

    // init empty prices
    for g in 0..n_groups {
        let mut group_map = HashMap::new();
        for w in 0..n_visits {
            let mut period_prices = Vec::new();
            for t in 0..n_periods {
                // Randomly select a parent for each price
                let random_parent = &individuals[rng.gen_range(0..individuals.len())];
                let price: f64 = random_parent.prices.0[&g][&w][t];
                period_prices.push(price);
            }
            group_map.insert(w, period_prices);
        }
        prices.insert(g, group_map);
    }
    
    Individual {
        prices: PriceMatrix(prices),
        ind_id,
        fitness_score: 0.0,
        simulation_result: SimulationResult {
            event_history: vec![],
            avg_regret: 0.0,
            regret: 0.0,
            customers: vec![],
            avg_time_sold_at: 0.0,
            n_sold: 0.0,
            revenue: 0.0,
        },
    }
}

pub fn evolve_pricing<'a>(
    run_id: i32,
    settings: &'a ProblemSettings,
    algorithm_settings: &ESSettings,
    mut writer: &mut csv::Writer<File>,
) -> (Individual<'a>, Individual<'a>) {
    // population as a vector of individuals.
    let mut population: Vec<Individual<'a>> = Vec::new();
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
            algorithm_settings.fn_evals
        ));
        ind_id += 1;

    }

    let best_individual = population.iter()
    .max_by(|a, b| a.fitness_score.partial_cmp(&b.fitness_score).unwrap())
    .unwrap();
    let mut best_solution = best_individual.clone();
    let mut best_score = best_individual.fitness_score;

    let mut success_count = 0;
    let mut prev_best_score = f64::NEG_INFINITY;

    let initial_best_solution = best_solution.clone();

    println!("Initial best score: {}", best_score);

    // iterate over generations

    let mut n_children = 0;
    let mut num_mutation_improved = 0;
    let mut num_recombination_improved = 0;
    for gen in 0..algorithm_settings.num_generations {
        let mut offspring: Vec<Individual<'a>> = Vec::new();
        let mut gen_best_solution = population[0].clone();
        let mut gen_best_score = 0.0;

        let mut avg_score = 0.0;

        println!("Population best score: {}, {}", best_score, gen_best_solution.fitness_score);

        // generate offspring
        for _ in 0..algorithm_settings.lambda {
            n_children += 1;
            let mut rng = rand::thread_rng();
            let mut parents = Vec::new();

            // mating selection
            for _ in 0..algorithm_settings.p {
                let parent_idx = rng.gen_range(0..algorithm_settings.mu);
                parents.push(population[parent_idx as usize].clone());
            }
            let mut offspring_individual = intermediate_recombination(parents.clone(), ind_id);
            
            let result = simulate_and_average(&mut offspring_individual, settings, algorithm_settings.fn_evals);
            offspring_individual.fitness_score = result.0;
            offspring_individual.simulation_result = result.1;

            // Check if offspring is better than parents
            let best_parent_score = parents.iter().map(|p| p.fitness_score).fold(f64::NEG_INFINITY, f64::max);
            if offspring_individual.fitness_score > best_parent_score {
                println!("Generation {}, Individual {}: Recombination improved fitness! {:.2} > {:.2}", 
                         gen, ind_id, offspring_individual.fitness_score, best_parent_score);
                num_recombination_improved += 1;
            }
            
            ind_id += 1;
            let mut mutated_offspring = mutate_solution(&offspring_individual, &params);

            n_evals += 1;
            let result = simulate_and_average(&mut mutated_offspring, settings, algorithm_settings.fn_evals);
            avg_score += result.0;

            mutated_offspring.fitness_score = result.0;
            mutated_offspring.simulation_result = result.1;

            // Check if mutation improved the individual
            if mutated_offspring.fitness_score > offspring_individual.fitness_score {
                println!("Generation {}, Individual {}: Mutation improved fitness! {:.2} > {:.2}", 
                         gen, ind_id-1, mutated_offspring.fitness_score, offspring_individual.fitness_score);
                num_mutation_improved += 1;
            }

            if mutated_offspring.fitness_score > gen_best_score {
                gen_best_score = mutated_offspring.fitness_score;
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
            &settings,
            n_evals,
            run_id,
        );
        log_population(
            &mut writer,
            &offspring,
            gen,
            "offspring",
            &params,
            &settings,
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

                println!("Success rate: {}, {}, {}", success_rate, params.mutation_strength, gen_best_score);

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

        if algorithm_settings.resample {
    // Reevaluate each individual in the population to prevent lucky solutions
            for individual in &mut population {
                let result = simulate_and_average(individual, settings, algorithm_settings.fn_evals);
                individual.fitness_score = result.0;
                individual.simulation_result = result.1;
            }
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
    println!(
        "Mutation improved: {}, Recombination improved: {}, Children: {}",
        num_mutation_improved, num_recombination_improved, n_children
    );
    // println!("Best overall revenue found: {}", best_score);
    (initial_best_solution, best_solution)
}
