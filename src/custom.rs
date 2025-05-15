use std::collections::HashMap;
use rand::Rng;
use crate::{ evolution::PriceMatrix, logging::log_event_history, mab::Algorithm, simulation::{simulate_revenue, ProblemSettings} };

pub struct CustomSolution {
    pub price_matrix: PriceMatrix,
}

impl CustomSolution {
    pub fn new(start_prices: Vec<f64>, settings: &ProblemSettings) -> Self {
        let mut rng = rand::thread_rng();
        let mut prices = HashMap::new();

        for (g, start_price) in start_prices.iter().enumerate() {
            let mut group_map = HashMap::new();
            for w in 0..settings.n_visits {
                let mut period_prices = Vec::new();
                // TODO: change back
                for _ in 0..settings.n_periods {
                    period_prices.push(*start_price);
                }
                group_map.insert(w as usize, period_prices);
            }
            prices.insert(g as usize, group_map);
        }

        let mut ind = Self {
            price_matrix: PriceMatrix(prices),
        };
        ind
    }
}

impl<'a> Algorithm for CustomSolution {
    fn get_price(&mut self, group_id: usize, visit: usize, period: usize) -> i32 {
        // Since ES maintains a price matrix with visits and periods,
        // we'll use the first visit and period for now
        // TODO: Extend the Algorithm trait to handle multiple visits/periods
        self.price_matrix.get_price(group_id, visit, period) as i32
    }

    fn update_average_reward(
        &mut self,
        _group_id: usize,
        _visit: usize,
        _period: usize,
        _reward: f64,
        _price: i32
    ) {
        // ES doesn't update prices based on individual rewards
        // Instead, it uses the total fitness score for evolution
    }
}

pub fn simulate_custom(settings: &ProblemSettings) {
    let vecA = vec![220.0, 550.0, 140.0];
    let vecB = vec![200.0, 500.0, 125.0];
    let n_runs = 100;
    
    let mut total_revenue_A = 0.0;
    let mut total_revenue_B = 0.0;
    
    println!("Running {} simulations for each price vector...", n_runs);

    let mut best_result_A = None;
    let mut best_result_B = None;
    let mut best_revenue_A= 0.0;
    let mut best_revenue_B= 0.0;
    
    for i in 0..n_runs {
        let mut custom_solution_A = CustomSolution::new(vecA.clone(), settings);
        let mut custom_solution_B = CustomSolution::new(vecB.clone(), settings);
        
        let res_A = simulate_revenue(&mut custom_solution_A, settings);
        let res_B = simulate_revenue(&mut custom_solution_B, settings);
        
        total_revenue_A += res_A.revenue as f64;
        total_revenue_B += res_B.revenue as f64;
        
        // println!("Run {}: Vector A: {:.2}, Vector B: {:.2}", 
        //     i + 1, 
        //     res_A.n_sold, 
        //     res_B.n_sold
        // );
        
        // Track the best result (highest revenue)
        if res_A.revenue as f64 > best_revenue_A {
            best_revenue_A = res_A.revenue as f64;
            best_result_A = Some(("A", res_A.clone()));
        }
        
        if res_B.revenue as f64 > best_revenue_B {
            best_revenue_B = res_B.revenue as f64;
            best_result_B = Some(("B", res_B.clone()));
        }
    }
    
    let avg_revenue_A = total_revenue_A / n_runs as f64;
    let avg_revenue_B = total_revenue_B / n_runs as f64;

    // Log the best result from all runs
    if let Some((vector_type, best)) = best_result_A {
        println!("\nBest result: Vector {} with revenue {:.2}", vector_type, best.revenue);
        log_event_history(0, &best, &settings);
    }

    if let Some((vector_type, best)) = best_result_B {
        println!("\nBest result: Vector {} with revenue {:.2}", vector_type, best.revenue);
        log_event_history(1, &best, &settings);
    }
    
    println!("\nResults after {} runs:", n_runs);
    println!("Vector A average revenue: {:.2}", avg_revenue_A);
    println!("Vector B average revenue: {:.2}", avg_revenue_B);
    println!("Difference (A - B): {:.2}", avg_revenue_A - avg_revenue_B);
}