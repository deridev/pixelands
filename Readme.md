# Bevy Particle Automata

A 2D cellular automata particle simulation inspired by classics like Powder Toy and Noita. Built with the [Bevy game engine](https://bevyengine.org/), this project allows you to experiment with different elements and observe their dynamic interactions in a procedurally generated, infinite world.

## Features

*   **Interactive Particle Simulation:** Watch as elements react to each other, gravity, and user input.
*   **Three Core Elements:**
    *   **Sand (Press `1`):** A classic falling particle that forms piles.
    *   **Water (Press `2`):** A flowing liquid that spreads out and seeks its own level.
    *   **Wall (Press `3`):** An immovable solid, perfect for creating boundaries and structures.
*   **Infinite Chunk System:** Simulate a virtually limitless world! The simulation space is managed by an efficient chunk-based system.
    *   **Dirty Rects Optimization:** Only modified areas of chunks are re-processed and re-rendered, significantly boosting performance.
*   **Acceleration-Based Particle Movement:** Particles don't just teleport; they accelerate due to gravity and other simulated forces, leading to more natural-looking motion, stacking, and flowing behaviors.

## Controls

### Camera
*   **`WASD`**: Move the camera view (pan up, left, down, right).
*   **`Mouse Scroll`**: Zoom in and out.

### Element Manipulation
*   **`1`**: Select Sand.
*   **`2`**: Select Water.
*   **`3`**: Select Wall.
*   **`Left Mouse Click`**: Place the selected element at the cursor's position.
*   **`Right Mouse Click`**: Remove an element at the cursor's position.

### Debugging
*   **`F1`**: Toggle the debug UI (may show performance metrics, chunk information, etc.).

## Technical Details

*   **Engine:** Built using the [Bevy Engine](https://bevyengine.org/), a modern, data-driven game engine written in Rust, chosen for its performance, modularity, and active community.
*   **Simulation Core:** A custom cellular automata engine where each pixel's state (element type, velocity, etc.) is updated based on its own properties and those of its neighbors. This creates emergent behaviors for the different elements.
*   **World Management:** Utilizes an "infinite" chunk system to dynamically process parts of the simulation space. Dirty rectangle tracking ensures that only regions with changes are updated, optimizing both simulation logic and rendering.

## Getting Started

### Prerequisites
*   **Rust:** Ensure you have Rust and Cargo installed. You can get them from [rustup.rs](https://rustup.rs/).
*   **System Dependencies:** Bevy has some [platform-specific dependencies](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md) (especially for Linux). Please check the Bevy documentation if you encounter build issues.

### Running the Simulation
1.  **Clone the repository:**
    ```bash
    git clone https://github.com/deridev/pixelands.git
    cd pixelands
    ```

2.  **Run the application:**
    For optimal performance, run in release mode:
    ```bash
    cargo run --release
    ```
    For quicker iteration during development (with less optimization):
    ```bash
    cargo run
    ```

## Future Ideas
This project is a foundation. Here are some potential features for the future:
*   More elements (e.g., fire, wood, steam, acid, gas).
*   Complex element interactions and chemical reactions.
*   Temperature simulation affecting element states.
*   Save/Load functionality for your creations.
*   Adjustable brush sizes and shapes for placing elements.
*   Sound effects for particle interactions.

## License
Distributed under the MIT License.