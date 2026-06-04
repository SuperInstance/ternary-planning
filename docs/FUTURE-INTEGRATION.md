# Future Integration: ternary-planning

## Current State
Provides priority-based task scheduling with ternary priorities (`TritAction`), resource allocation with ternary constraints (reserve/flexible/free), dependency graph execution (`PlanGraph`), and constraint satisfaction planning.

## Integration Opportunities

### With ternary-cell / room-as-codespace
Room transitions are planned tasks. A `TernaryTask` represents a room state change with priority (urgent/normal/deferrable). `PlanGraph` sequences room transitions respecting dependencies (e.g., "cool room A before moving perishable items from room B to room A"). `ResourceAllocation` manages shared resources (HVAC capacity, power budget) across multiple rooms.

### With ternary-graph
`PlanGraph` is internally a DAG. `ternary-graph`'s traversal algorithms (BFS, DFS, shortest path) can optimize plan execution order. When dependencies have ternary weights (strong/preferred/weak), `ternary-graph::shortest_path()` finds the execution plan with strongest dependency satisfaction.

### With ternary-rl
Planning and RL form a hierarchy: `ternary-planning` generates high-level plans (which rooms to visit in what order), `ternary-rl` learns low-level execution (what actions to take within each room). The `TernaryTask::priority` guides RL exploration — high-priority tasks get more learning budget.

## Potential in Mature Systems
In PLATO's ensign system, each ensign receives a `PlanGraph` for its shift. The ensign decomposes the plan into individual `TernaryTask`s, executes them via `PriorityScheduler`, and reports completion via `ternary-protocol`. `ResourceAllocation::allocate()` prevents two ensigns from competing for the same resource. At Layer 0, the plan collapses to a simple priority queue — no graph needed, just a sorted task list.

## Cross-Pollination Ideas
**Music × Planning:** A musical performance is a plan. Each measure is a `TernaryTask` with priority (downbeat = Positive, pickup = Zero, passing = Negative). The conductor is the `PriorityScheduler`. Voice leading between measures creates `PlanGraph` dependencies. `ternary-music`'s harmonic constraints become planning constraints.

**Game theory × Planning:** Multi-agent planning with `ternary-game-theory`: agents negotiate plan priority via coalition formation. Tasks with Positive priority form coalitions; Zero tasks are independent; Negative tasks are adversarial.

## Dependencies for Next Steps
- Integration with `ternary-graph` for plan optimization
- Real-time replanning when sensor input invalidates a plan
- `ternary-protocol` messages for plan distribution across constructs
