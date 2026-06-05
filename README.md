# ternary-planning

Planning and scheduling with ternary priorities — priority-based task scheduling, resource-constrained allocation, dependency graphs, and constraint-satisfaction solvers over {-1, 0, +1} priorities.

## Why This Exists

Scheduling systems typically use numeric priority levels or weight-based heuristics. But many real-world decisions naturally collapse to three levels: must-do / optional / skip, critical / normal / low, or approve / abstain / reject. This crate models tasks with ternary priorities and provides schedulers, resource allocators, dependency-aware plan graphs, and both greedy and brute-force constraint-satisfaction solvers that work natively with three-valued priorities. `forbid(unsafe_code)` throughout.

## Core Concepts

- **TritAction**: Ternary priority — `Negative` (-1), `Zero` (0), `Positive` (+1).
- **TernaryTask**: A task with id, name, ternary priority, duration, and resource requirement.
- **ResourceAllocation**: Resource pool with ternary constraints — `Positive` reserves 30% (conservative), `Zero` is flexible, `Negative` is fully available.
- **PriorityScheduler**: Orders tasks by priority (Positive → Zero → Negative), optionally respecting resource constraints.
- **PlanGraph**: DAG of tasks with dependencies, topological sort, and cycle/conflict detection.
- **greedy_solve**: Priority-first greedy packing under resource limits.
- **csp_solve**: Brute-force constraint-satisfaction solver that maximizes total priority score (suitable for small task sets).

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-planning = "0.1"
```

```rust
use ternary_planning::{
    TritAction, TernaryTask, PriorityScheduler, ResourceAllocation,
    PlanGraph, greedy_solve, csp_solve,
};

fn main() {
    // Create tasks with ternary priorities
    let tasks = vec![
        TernaryTask::new(0, "critical feature",  TritAction::Positive, 5, 30),
        TernaryTask::new(1, "nice-to-have",      TritAction::Zero,     3, 20),
        TernaryTask::new(2, "known issue",        TritAction::Negative, 2, 10),
    ];

    // Priority scheduling
    let mut scheduler = PriorityScheduler::new();
    for t in &tasks {
        scheduler.add_task(t.clone());
    }
    let order: Vec<usize> = scheduler.schedule().iter().map(|t| t.id).collect();
    assert_eq!(order, vec![0, 1, 2]); // Positive first

    // Resource-constrained scheduling
    let mut resource = ResourceAllocation::new(100, TritAction::Zero);
    let scheduled = scheduler.schedule_with_resources(&mut resource);

    // Dependency graph
    let mut graph = PlanGraph::new();
    let a = graph.add_task(TernaryTask::new(0, "design", TritAction::Positive, 1, 1), vec![]);
    let b = graph.add_task(TernaryTask::new(0, "implement", TritAction::Positive, 1, 1), vec![a]);
    let c = graph.add_task(TernaryTask::new(0, "test", TritAction::Zero, 1, 1), vec![a, b]);
    let order = graph.topological_order().unwrap();

    // Solvers
    let greedy_result = greedy_solve(&tasks, 50);
    let optimal_result = csp_solve(&tasks, 50);
}
```

## API Overview

| Type / Function | Description |
|---|---|
| `TritAction` | Ternary priority: `Negative`, `Zero`, `Positive` |
| `TernaryTask` | Task with priority, duration, resource need |
| `ResourceAllocation` | Resource pool with ternary reservation constraint |
| `PriorityScheduler` | Priority-ordered scheduling, with optional resource constraints |
| `PlanGraph` | DAG of tasks with `topological_order()`, `detect_conflicts()` |
| `greedy_solve` | Greedy priority-first packing under resource limit |
| `csp_solve` | Brute-force optimal selection maximizing total priority |

## How It Works

**PriorityScheduler** sorts tasks by their ternary priority value (Positive=1 first, Negative=-1 last). When resources are involved, it allocates greedily in priority order, skipping tasks that exceed available resources.

**ResourceAllocation** manages a pool with three modes. `Positive` constraint reserves 30% of total capacity (conservative mode). `Zero` is fully flexible. `Negative` makes everything available — no reservation.

**PlanGraph** stores tasks as nodes in a DAG with explicit dependency lists. `topological_order()` uses Kahn's algorithm (in-degree counting with a queue). `detect_conflicts()` uses DFS cycle detection and returns conflicting edge pairs.

**csp_solve** enumerates all `2ⁿ` subsets of tasks, sums resource requirements and priority scores, and returns the subset with maximum total priority that fits within the resource budget. Suitable for n ≤ ~20 tasks.

## Use Cases

- **Sprint planning**: Prioritize backlog items as must-do / optional / skip, schedule under team capacity constraints.
- **Resource-constrained project management**: Model projects with ternary priority tasks and limited resources (budget, personnel, compute).
- **Dependency-aware task scheduling**: Build plan graphs for multi-stage workflows with dependencies and detect circular dependencies.
- **Incident triage**: Classify incidents as critical / normal / low priority and allocate response resources accordingly.

## Ecosystem

Part of the **SuperInstance** ternary computing suite:

- `ternary-lattice` — lattice structures for ternary values
- `ternary-codes` — error-correcting codes for ternary data
- `ternary-gradient` — gradient-free optimization on ternary landscapes
- `ternary-language` — ternary NLP and grammar processing
- `ternary-trees` — ternary decision trees and forests
- `ternary-transform` — wavelet, Fourier, and kernel transforms
- `ternary-planning` — this crate
- `ternary-rl` — reinforcement learning with ternary actions
- `ternary-som` — self-organizing maps for ternary data
- `ternary-failure` — failure analysis with ternary classification

## License

MIT

## See Also
- **ternary-constraint** — related
- **ternary-scheduling** — related
- **ternary-search** — related
- **ternary-optimization** — related
- **ternary-control** — related

