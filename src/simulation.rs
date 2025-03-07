use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use rand::{rngs::ThreadRng, Rng};
use rand_distr::{Exp, Normal};
use std::cmp::Reverse;
use std::collections::HashMap;


#[derive(Debug)]
pub struct Customer<'a> {
    id: i32,
    group: i32,
    irp: f64,
    erp: f64,
    rp: f64,
    wtp: f64,
    price_hist: Vec<f64>,
    visit_hist: Vec<i32>,
    settings: &'a ProblemSettings,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct SimulationEvent {
    t: OrderedFloat<f32>,
    event: String,
    customer: i32,
}

impl<'a> Customer<'a> {
    pub fn new(id: i32, group: i32, wtp_min: f64, wtp_max: f64, settings: &'a ProblemSettings) -> Self {
        let mut rng = rand::thread_rng();
        let wtp: f64 = rng.gen_range(wtp_min..=wtp_max);
        Customer {
            id,
            group,
            irp: -1.0,
            erp: -1.0,
            rp: -1.0,
            wtp,
            price_hist: vec![],
            visit_hist: vec![],
            settings,
        }
    }
    pub fn update_irp(&mut self, new_price: f64) {
        self.irp = self.irp + self.settings.tau * (new_price - self.irp)
    }
    pub fn update_erp(&mut self, other_customers: Vec<&Customer>) {
        let mut rng = rand::thread_rng();
        let mut ref_prices: Vec<f64> = vec![];
        for cust in other_customers {
            if rng.gen::<f64>() < 0.2 {
                if let Some(&price) = cust.price_hist.last() {
                    ref_prices.push(price);
                }
            }
        }
    }
    pub fn update_rp(&mut self) {}
    pub fn update_wtp(&mut self) {}
    pub fn next_visit(&self, rng: &mut ThreadRng, t: f32) -> f32 {
        let distr = Exp::new(0.1).unwrap();
        return t + rng.sample::<f32, _>(distr);
    }
}

// Move ProblemSettings, Customer, SimulationEvent, and Solution here
#[derive(Debug)]
pub struct ProblemSettings {
    pub n_visits: i32,
    pub n_periods: i32,
    pub n_groups: i32,
    pub n_customers: i32,
    pub tau: f64,
    pub scaling: f64,
    pub group_sizes: Vec<i32>,
    pub group_means: Vec<f64>,
}

#[derive(Clone)]
pub struct Solution {
    pub prices: HashMap<i32, HashMap<i32, Vec<f64>>>,
}

impl Solution {
    pub fn new(n_visits: i32, n_periods: i32, n_groups: i32) -> Self {
        let mut rng = rand::thread_rng();
        let mut prices = HashMap::new();

        for w in 0..n_visits {
            let mut group_map = HashMap::new();
            for g in 0..n_groups {
                let mut period_prices = Vec::new();
                for _ in 0..n_periods {
                    period_prices.push(rng.gen_range(0.0..100.0));
                }
                group_map.insert(g, period_prices);
            }
            prices.insert(w, group_map);
        }

        Self { prices }
    }

    fn get_price(&self, w: i32, g: i32, t: i32) -> f64 {
        self.prices[&w][&g][t as usize]
    }
}

pub fn init_simulation(
    customers: &Vec<Customer>,
) -> PriorityQueue<SimulationEvent, Reverse<OrderedFloat<f32>>> {
    let mut rng = rand::thread_rng();

    let mut event_calendar: PriorityQueue<SimulationEvent, Reverse<OrderedFloat<f32>>> =
        PriorityQueue::new();

    for customer in customers {
        let event = SimulationEvent {
            t: OrderedFloat(customer.next_visit(&mut rng, 0.0)),
            event: "customer_arrival".to_string(),
            customer: customer.id,
        };
        event_calendar.push(event.clone(), Reverse(event.t));
    }

    return event_calendar;
}

// do one simulation run
pub fn simulate_revenue(solution: &Solution, settings: &ProblemSettings) -> f64 {
    let mut rng = rand::thread_rng();

    let mut customers: Vec<Customer> = Vec::new();
    let mut id = 0;
    for i in 0..settings.group_sizes.len() {
        let group_size = settings.group_sizes[i];
        let group_mean = settings.group_means[i];
        for _ in 0..group_size {
            let normal_dist = Normal::new(group_mean * settings.scaling, 5.0).unwrap();
            let wtp0: f64 = rng.sample(normal_dist);
            customers.push(Customer::new(id, i as i32, wtp0, wtp0, settings));
            id += 1;
        }
    }

    let mut event_calendar = init_simulation(&customers);
    let mut revenue = 0.0;
    let max_events = 1000;
    let mut event_count = 0;
    while !event_calendar.is_empty() && event_count < max_events {
        event_count += 1;
        let event = event_calendar.pop();
        if event.is_none() {
            break;
        }
        let event = event.unwrap();
        if event.0.t > OrderedFloat(settings.n_periods as f32) {
            continue;
        }
        let customer_idx = event.0.customer as usize;
        let next_visit = customers[customer_idx].next_visit(&mut rng, event.0.t.0);
        let visit_index = customers[customer_idx].price_hist.len() as i32;
        let price = solution.get_price(
            visit_index,
            customers[customer_idx].group,
            event.0.t.0 as i32,
        );
        if price < customers[customer_idx].wtp {
            revenue += price;
            customers[customer_idx].price_hist.push(price);
        } else {
            let next_event = SimulationEvent {
                t: OrderedFloat(next_visit),
                event: "customer_arrival".to_string(),
                customer: event.0.customer,
            };
            event_calendar.push(next_event, Reverse(OrderedFloat(next_visit)));
        }
    }

    revenue
}