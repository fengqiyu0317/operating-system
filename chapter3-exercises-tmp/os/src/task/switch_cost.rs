//! Task switching cost statistics

use crate::timer::get_time;
use crate::sync::UPSafeCell;
use lazy_static::*;

/// Task switching cost statistics
#[derive(Default, Debug)]
pub struct SwitchCostStats {
    /// Total number of task switches
    pub total_switches: usize,
    /// Total cost of all switches (in CPU cycles)
    pub total_cost: usize,
    /// Maximum single switch cost
    pub max_cost: usize,
    /// Minimum single switch cost
    pub min_cost: usize,
}

impl SwitchCostStats {
    /// Record a switch cost
    pub fn record(&mut self, cost: usize) {
        self.total_switches += 1;
        self.total_cost += cost;
        if self.total_switches <= 20 {
            println!("[kernel] Switch #{} cost: {} cycles", self.total_switches, cost);
        }
        if cost > self.max_cost {
            self.max_cost = cost;
        }
        if self.min_cost == 0 || cost < self.min_cost {
            self.min_cost = cost;
        }
    }

    /// Get average switch cost
    pub fn average_cost(&self) -> usize {
        if self.total_switches == 0 {
            0
        } else {
            self.total_cost / self.total_switches
        }
    }

    /// Print statistics
    pub fn print(&self) {
        if self.total_switches > 0 {
            println!("[kernel] Task Switch Statistics:");
            println!("  Total switches: {}", self.total_switches);
            println!("  Total cost: {} cycles", self.total_cost);
            println!("  Average cost: {} cycles", self.average_cost());
            println!("  Max cost: {} cycles", self.max_cost);
            println!("  Min cost: {} cycles", self.min_cost);
        }
    }
}

/// Global switch cost start time (stored before __switch)
static mut SWITCH_START_TIME: usize = 0;

// Global switch cost statistics
lazy_static! {
    pub static ref SWITCH_COST_STATS: UPSafeCell<SwitchCostStats> =
        unsafe { UPSafeCell::new(SwitchCostStats::default()) };
}

/// Mark the start time before switching
#[inline]
pub fn mark_switch_start() {
    unsafe {
        SWITCH_START_TIME = get_time();
    }
}

/// Record the switch cost after returning from switch
/// Returns the recorded cost
#[inline]
pub fn record_switch_cost() -> usize {
    let end_time = get_time();
    unsafe {
        let cost = end_time - SWITCH_START_TIME;
        SWITCH_COST_STATS.exclusive_access().record(cost);
        cost
    }
}

/// Print and get the statistics
pub fn print_switch_stats() {
    SWITCH_COST_STATS.exclusive_access().print();
}
