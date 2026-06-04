#![forbid(unsafe_code)]

//! Planning and scheduling with ternary priorities.

use std::collections::{HashMap, HashSet, VecDeque};

/// Ternary action/trit: Negative (-1), Zero (0), Positive (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TritAction {
    Negative = -1,
    Zero = 0,
    Positive = 1,
}

impl TritAction {
    pub fn value(&self) -> i32 {
        *self as i32
    }

    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            -1 => Some(TritAction::Negative),
            0 => Some(TritAction::Zero),
            1 => Some(TritAction::Positive),
            _ => None,
        }
    }
}

/// A task with a ternary priority.
#[derive(Debug, Clone)]
pub struct TernaryTask {
    pub id: usize,
    pub name: String,
    pub priority: TritAction,
    pub duration: u64,
    pub resource_need: u64,
}

impl TernaryTask {
    pub fn new(id: usize, name: &str, priority: TritAction, duration: u64, resource_need: u64) -> Self {
        Self { id, name: name.to_string(), priority, duration, resource_need }
    }
}

/// Resource allocation with ternary constraints.
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub total: u64,
    pub allocated: u64,
    pub constraint: TritAction, // Positive = must reserve, Zero = flexible, Negative = free
}

impl ResourceAllocation {
    pub fn new(total: u64, constraint: TritAction) -> Self {
        Self { total, allocated: 0, constraint }
    }

    pub fn available(&self) -> u64 {
        self.total.saturating_sub(self.allocated)
    }

    pub fn allocate(&mut self, amount: u64) -> bool {
        let reserved = match self.constraint {
            TritAction::Positive => (self.total as f64 * 0.3) as u64,
            TritAction::Zero => 0,
            TritAction::Negative => 0,
        };
        if amount <= self.available().saturating_sub(reserved) {
            self.allocated += amount;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self, amount: u64) {
        self.allocated = self.allocated.saturating_sub(amount);
    }
}

/// Priority-based scheduler.
pub struct PriorityScheduler {
    tasks: Vec<TernaryTask>,
}

impl PriorityScheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: TernaryTask) {
        self.tasks.push(task);
    }

    /// Schedule tasks in priority order (Positive first, then Zero, then Negative).
    pub fn schedule(&self) -> Vec<&TernaryTask> {
        let mut sorted: Vec<&TernaryTask> = self.tasks.iter().collect();
        sorted.sort_by(|a, b| b.priority.value().cmp(&a.priority.value()));
        sorted
    }

    /// Schedule with resource constraints.
    pub fn schedule_with_resources(&self, resource: &mut ResourceAllocation) -> Vec<&TernaryTask> {
        let sorted = self.schedule();
        let mut result = Vec::new();
        for task in sorted {
            if resource.allocate(task.resource_need) {
                result.push(task);
            }
        }
        result
    }
}

/// A node in a PlanGraph.
#[derive(Debug, Clone)]
pub struct PlanNode {
    pub task: TernaryTask,
    pub dependencies: Vec<usize>,
}

/// A directed acyclic graph of ternary-weighted tasks.
pub struct PlanGraph {
    nodes: HashMap<usize, PlanNode>,
    next_id: usize,
}

impl PlanGraph {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), next_id: 0 }
    }

    pub fn add_task(&mut self, task: TernaryTask, deps: Vec<usize>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let node = PlanNode { task: TernaryTask { id, ..task }, dependencies: deps };
        self.nodes.insert(id, node);
        id
    }

    pub fn get(&self, id: usize) -> Option<&PlanNode> {
        self.nodes.get(&id)
    }

    /// Topological sort respecting dependencies.
    pub fn topological_order(&self) -> Option<Vec<usize>> {
        let mut in_degree: HashMap<usize, usize> = HashMap::new();
        for id in self.nodes.keys() {
            in_degree.insert(*id, 0);
        }
        for node in self.nodes.values() {
            for &dep in &node.dependencies {
                *in_degree.entry(node.task.id).or_insert(0) += if self.nodes.contains_key(&dep) { 0 } else { 0 };
            }
            // Count how many nodes depend on each node
        }
        // Recompute: in_degree[x] = number of nodes that list x as dependency... no.
        // in_degree[x] = number of dependencies of x that are in the graph
        for node in self.nodes.values() {
            let count = node.dependencies.iter().filter(|d| self.nodes.contains_key(d)).count();
            in_degree.insert(node.task.id, count);
        }

        let mut queue: VecDeque<usize> = {
            let mut v: Vec<usize> = in_degree.iter()
                .filter(|(_, &deg)| deg == 0)
                .map(|(&id, _)| id)
                .collect();
            v.sort();
            v.into_iter().collect()
        };

        let mut result = Vec::new();
        while let Some(id) = queue.pop_front() {
            result.push(id);
            for node in self.nodes.values() {
                if node.dependencies.contains(&id) {
                    let deg = in_degree.get_mut(&node.task.id).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(node.task.id);
                    }
                }
            }
        }

        if result.len() == self.nodes.len() {
            Some(result)
        } else {
            None
        }
    }

    /// Detect if the graph has cycles (conflicts in dependencies).
    pub fn detect_conflicts(&self) -> HashSet<(usize, usize)> {
        let mut conflicts = HashSet::new();
        // Check for cycles using DFS
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        for &id in self.nodes.keys() {
            if !visited.contains(&id) {
                self.detect_cycles_dfs(id, &mut visited, &mut stack, &mut conflicts);
            }
        }
        conflicts
    }

    fn detect_cycles_dfs(&self, id: usize, visited: &mut HashSet<usize>, stack: &mut HashSet<usize>, conflicts: &mut HashSet<(usize, usize)>) {
        visited.insert(id);
        stack.insert(id);

        if let Some(node) = self.nodes.get(&id) {
            for &dep in &node.dependencies {
                if !self.nodes.contains_key(&dep) {
                    continue;
                }
                if stack.contains(&dep) {
                    conflicts.insert((id, dep));
                } else if !visited.contains(&dep) {
                    self.detect_cycles_dfs(dep, visited, stack, conflicts);
                }
            }
        }

        stack.remove(&id);
    }
}

/// Greedy solver: picks highest priority first, respecting resources.
pub fn greedy_solve(tasks: &[TernaryTask], total_resources: u64) -> Vec<usize> {
    let mut resource = ResourceAllocation::new(total_resources, TritAction::Zero);
    let mut sorted: Vec<&TernaryTask> = tasks.iter().collect();
    sorted.sort_by(|a, b| b.priority.value().cmp(&a.priority.value()));

    let mut result = Vec::new();
    for task in sorted {
        if resource.allocate(task.resource_need) {
            result.push(task.id);
        }
    }
    result
}

/// Constraint-satisfaction solver: tries to maximize total priority score.
pub fn csp_solve(tasks: &[TernaryTask], total_resources: u64) -> Vec<usize> {
    let n = tasks.len();
    let mut best_score = i32::MIN;
    let mut best_set: Vec<usize> = Vec::new();

    // Brute force for small task sets
    for mask in 0..(1u32 << n) {
        let mut total_res = 0u64;
        let mut score = 0i32;
        let mut selected = Vec::new();
        for i in 0..n {
            if mask & (1 << i) != 0 {
                total_res += tasks[i].resource_need;
                score += tasks[i].priority.value();
                selected.push(tasks[i].id);
            }
        }
        if total_res <= total_resources && score > best_score {
            best_score = score;
            best_set = selected;
        }
    }
    best_set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trit_action_values() {
        assert_eq!(TritAction::Negative.value(), -1);
        assert_eq!(TritAction::Zero.value(), 0);
        assert_eq!(TritAction::Positive.value(), 1);
    }

    #[test]
    fn test_trit_from_i32() {
        assert_eq!(TritAction::from_i32(-1), Some(TritAction::Negative));
        assert_eq!(TritAction::from_i32(0), Some(TritAction::Zero));
        assert_eq!(TritAction::from_i32(1), Some(TritAction::Positive));
        assert_eq!(TritAction::from_i32(2), None);
    }

    #[test]
    fn test_task_creation() {
        let t = TernaryTask::new(1, "test", TritAction::Positive, 5, 10);
        assert_eq!(t.id, 1);
        assert_eq!(t.name, "test");
        assert_eq!(t.duration, 5);
    }

    #[test]
    fn test_resource_allocation_basic() {
        let mut r = ResourceAllocation::new(100, TritAction::Zero);
        assert!(r.allocate(50));
        assert_eq!(r.available(), 50);
        assert!(!r.allocate(60));
    }

    #[test]
    fn test_resource_release() {
        let mut r = ResourceAllocation::new(100, TritAction::Zero);
        r.allocate(80);
        r.release(30);
        assert_eq!(r.available(), 50);
    }

    #[test]
    fn test_resource_reserved_constraint() {
        let mut r = ResourceAllocation::new(100, TritAction::Positive);
        // 30% reserved = 30, so only 70 available
        assert!(r.allocate(70));
        assert!(!r.allocate(1));
    }

    #[test]
    fn test_scheduler_priority_order() {
        let mut s = PriorityScheduler::new();
        s.add_task(TernaryTask::new(0, "low", TritAction::Negative, 1, 1));
        s.add_task(TernaryTask::new(1, "high", TritAction::Positive, 1, 1));
        s.add_task(TernaryTask::new(2, "mid", TritAction::Zero, 1, 1));
        let order: Vec<usize> = s.schedule().iter().map(|t| t.id).collect();
        assert_eq!(order, vec![1, 2, 0]);
    }

    #[test]
    fn test_scheduler_with_resources() {
        let mut s = PriorityScheduler::new();
        s.add_task(TernaryTask::new(0, "a", TritAction::Positive, 1, 60));
        s.add_task(TernaryTask::new(1, "b", TritAction::Zero, 1, 50));
        s.add_task(TernaryTask::new(2, "c", TritAction::Negative, 1, 30));
        let mut res = ResourceAllocation::new(100, TritAction::Zero);
        let scheduled: Vec<usize> = s.schedule_with_resources(&mut res).iter().map(|t| t.id).collect();
        assert_eq!(scheduled, vec![0, 2]); // a(60) + c(30) = 90 <= 100, b(50) won't fit
    }

    #[test]
    fn test_plan_graph_add() {
        let mut g = PlanGraph::new();
        let id = g.add_task(TernaryTask::new(0, "root", TritAction::Positive, 1, 1), vec![]);
        assert!(g.get(id).is_some());
    }

    #[test]
    fn test_plan_graph_topo_sort() {
        let mut g = PlanGraph::new();
        let a = g.add_task(TernaryTask::new(0, "a", TritAction::Positive, 1, 1), vec![]);
        let b = g.add_task(TernaryTask::new(0, "b", TritAction::Zero, 1, 1), vec![a]);
        let c = g.add_task(TernaryTask::new(0, "c", TritAction::Negative, 1, 1), vec![a, b]);
        let order = g.topological_order().unwrap();
        assert!(order.iter().position(|&x| x == a).unwrap() < order.iter().position(|&x| x == b).unwrap());
        assert!(order.iter().position(|&x| x == b).unwrap() < order.iter().position(|&x| x == c).unwrap());
    }

    #[test]
    fn test_plan_graph_cycle_detection() {
        let mut g = PlanGraph::new();
        let a = g.add_task(TernaryTask::new(0, "a", TritAction::Positive, 1, 1), vec![]);
        let b = g.add_task(TernaryTask::new(0, "b", TritAction::Zero, 1, 1), vec![a]);
        // Manually create cycle by adding b as dep of a
        if let Some(node) = g.nodes.get_mut(&a) {
            node.dependencies.push(b);
        }
        let conflicts = g.detect_conflicts();
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_plan_graph_no_conflicts() {
        let mut g = PlanGraph::new();
        let a = g.add_task(TernaryTask::new(0, "a", TritAction::Positive, 1, 1), vec![]);
        g.add_task(TernaryTask::new(0, "b", TritAction::Zero, 1, 1), vec![a]);
        assert!(g.detect_conflicts().is_empty());
    }

    #[test]
    fn test_greedy_solver() {
        let tasks = vec![
            TernaryTask::new(0, "low", TritAction::Negative, 1, 10),
            TernaryTask::new(1, "high", TritAction::Positive, 1, 30),
            TernaryTask::new(2, "mid", TritAction::Zero, 1, 20),
        ];
        let result = greedy_solve(&tasks, 60);
        assert!(result.contains(&1)); // high priority always included
    }

    #[test]
    fn test_csp_solver_optimal() {
        let tasks = vec![
            TernaryTask::new(0, "a", TritAction::Positive, 1, 60),
            TernaryTask::new(1, "b", TritAction::Positive, 1, 60),
            TernaryTask::new(2, "c", TritAction::Negative, 1, 10),
        ];
        let result = csp_solve(&tasks, 70);
        // Both a and b can't fit; csp picks best score
        assert!(result.contains(&0) || result.contains(&1));
    }

    #[test]
    fn test_empty_scheduler() {
        let s = PriorityScheduler::new();
        assert!(s.schedule().is_empty());
    }

    #[test]
    fn test_empty_plan_graph() {
        let g = PlanGraph::new();
        assert_eq!(g.topological_order(), Some(vec![]));
    }

    #[test]
    fn test_resource_zero_total() {
        let mut r = ResourceAllocation::new(0, TritAction::Zero);
        assert!(!r.allocate(1));
    }

    #[test]
    fn test_greedy_no_resources() {
        let tasks = vec![TernaryTask::new(0, "a", TritAction::Positive, 1, 10)];
        let result = greedy_solve(&tasks, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_csp_empty() {
        let result = csp_solve(&[], 100);
        assert!(result.is_empty());
    }

    #[test]
    fn test_resource_negative_constraint() {
        let mut r = ResourceAllocation::new(100, TritAction::Negative);
        // Negative constraint = no reservation, all available
        assert!(r.allocate(100));
        assert!(!r.allocate(1));
    }

    #[test]
    fn test_plan_graph_missing_dependency() {
        let mut g = PlanGraph::new();
        g.add_task(TernaryTask::new(0, "a", TritAction::Positive, 1, 1), vec![999]);
        let order = g.topological_order();
        assert!(order.is_some()); // Missing dep ignored
    }
}
