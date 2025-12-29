# Physics Simulations in Rust

A collection of physics simulations implemented in Rust using [wgpu](https://wgpu.rs/) for GPU-accelerated rendering. These are Rust reimplementations of the C++ physics simulations by [kavan010](https://github.com/kavan010).

## Simulations

### 1. Gravity Simulation (`gravity_sim`)

N-body gravitational simulation using Newtonian physics.

**Features:**
- Real-time N-body gravitational interactions
- Multiple presets: Solar System, Accretion Disk, Galaxy Collision
- Leapfrog integration for stability
- Interactive camera controls

**Run:**
```bash
cargo run -p gravity_sim
# Or 3D version:
cargo run -p gravity_sim --bin gravity_sim_3d
```

**Controls:**
- `1/2/3` - Load presets (Solar System, Disk, Galaxy Collision)
- `Scroll` - Zoom in/out
- `Arrow keys / WASD` - Pan camera
- `Space` - Pause/Resume
- `R` - Reset

---

### 2. Black Hole Simulation (`black_hole`)

Schwarzschild black hole visualization with gravitational lensing.

**Features:**
- Ray-traced gravitational lensing
- Event horizon and photon sphere visualization
- Accretion disk with temperature-based coloring
- Star field background
- RK4 integration for geodesic equations

**Run:**
```bash
# 2D gravitational lensing visualization
cargo run -p black_hole --bin black_hole_2d

# 3D ray-marched visualization
cargo run -p black_hole
```

**Controls:**
- `Click` - Spawn light rays from position
- `+/-` - Adjust black hole mass
- `Scroll` - Zoom in/out
- `Arrow keys` - Pan camera
- `Space` - Toggle continuous ray emission (2D) / Pause animation (3D)
- `R` - Reset

---

### 3. Atoms Simulation (`atoms`)

Molecular dynamics simulation with atomic interactions.

**Features:**
- Coulomb forces for charged particles
- Lennard-Jones potential for van der Waals interactions
- Dynamic covalent bond formation
- Multiple molecule presets (H2O, NaCl, CH4)
- Element-specific properties and colors

**Run:**
```bash
cargo run -p atoms
```

**Controls:**
- `1/2/3/4` - Load presets (Water, Salt, Organic, Random)
- `H/C/N/O` - Select element for placing
- `Click` - Place atom at cursor
- `G` - Toggle grid
- `T` - Heat up (decrease damping)
- `Shift+T` - Cool down (increase damping)
- `Space` - Pause/Resume
- `R` - Reset

---

## Building

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- A GPU with Vulkan, Metal, DX12, or WebGPU support

### Build All

```bash
cargo build --release
```

### Build Specific Simulation

```bash
cargo build -p gravity_sim --release
cargo build -p black_hole --release
cargo build -p atoms --release
```

## Project Structure

```
.
├── Cargo.toml          # Workspace configuration
├── common/             # Shared graphics and utility code
│   └── src/
│       ├── lib.rs
│       ├── graphics.rs # wgpu setup and vertex types
│       └── camera.rs   # 2D/3D camera systems
├── gravity_sim/        # N-body gravity simulation
│   └── src/
│       ├── main.rs     # 2D simulation
│       ├── main_3d.rs  # 3D simulation
│       ├── physics.rs  # Gravitational physics
│       └── renderer.rs # Particle rendering
├── black_hole/         # Black hole visualization
│   └── src/
│       ├── main.rs     # 3D ray-marching
│       ├── main_2d.rs  # 2D lensing visualization
│       ├── physics.rs  # Schwarzschild geodesics
│       └── renderer.rs # Ray tracing renderer
└── atoms/              # Molecular dynamics
    └── src/
        ├── main.rs     # Main simulation
        ├── physics.rs  # Atomic forces and bonding
        └── renderer.rs # Atom/bond rendering
```

## Physics Concepts

### Gravity Simulation
- **Newton's Law of Gravitation**: F = G * m1 * m2 / r²
- **Leapfrog Integration**: Symplectic integrator for orbital mechanics
- **Softening Parameter**: Prevents numerical instabilities at close approaches

### Black Hole Simulation
- **Schwarzschild Radius**: rs = 2GM/c² (event horizon)
- **Photon Sphere**: r = 1.5 * rs (unstable photon orbits)
- **Geodesic Equations**: Light paths through curved spacetime
- **Runge-Kutta 4 (RK4)**: Numerical integration of geodesics

### Atoms Simulation
- **Coulomb's Law**: F = k * q1 * q2 / r² (electrostatic forces)
- **Lennard-Jones Potential**: V(r) = 4ε[(σ/r)¹² - (σ/r)⁶] (van der Waals)
- **Covalent Bonding**: Simplified spring model for molecular bonds

## Credits

Original C++ implementations:
- [kavan010/gravity_sim](https://github.com/kavan010/gravity_sim)
- [kavan010/black_hole](https://github.com/kavan010/black_hole)
- [kavan010/Atoms](https://github.com/kavan010/Atoms)

YouTube explanations:
- [Simulating a Black Hole](https://www.youtube.com/watch?v=8-B6ryuBkCM)
- [Gravity Simulation](https://www.youtube.com/watch?v=_YbGWoUaZg0)

## License

MIT License
