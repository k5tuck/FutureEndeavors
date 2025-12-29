# Physics Simulations in Rust

A collection of physics simulations implemented in Rust using [wgpu](https://wgpu.rs/) for GPU-accelerated rendering. These are Rust reimplementations of the C++ physics simulations by [kavan010](https://github.com/kavan010).

## Simulations

### 1. Solar Voyage (`solar_voyage`) - NEW!

An integrated interstellar journey simulation combining the solar system, black holes, and a controllable spaceship with relativistic effects.

**Features:**
- Accurate solar system with real planetary orbital data
- Controllable spaceship with thrust and rotation
- Relativistic effects (time dilation, Lorentz factor, Doppler shift)
- Optional black hole with gravitational effects
- Spacetime curvature visualization (rubber-sheet model)
- Star field with relativistic aberration
- Multiple camera modes (orbit, follow ship, first-person)

**Run:**
```bash
cargo run -p solar_voyage
```

**Controls:**
| Key | Action |
|-----|--------|
| `W/S` | Thrust forward/backward |
| `A/D` | Yaw left/right |
| `R/F` | Pitch up/down |
| `Q/E` | Roll left/right |
| `Shift` | Boost thrust |
| `Tab` | Toggle camera mode |
| `Space` | Pause/Resume |
| `G` | Toggle spacetime grid |
| `T` | Toggle orbital trails |
| `B` | Add/Remove black hole |
| `0-3` | Focus on celestial body |
| `+/-` | Adjust time scale |
| `Scroll` | Zoom |
| `Drag` | Orbit camera |

---

### 2. Gravity Simulation (`gravity_sim`)

N-body gravitational simulation using Newtonian physics. Now with full 3D support!

**Features:**
- Real-time N-body gravitational interactions
- Full 3D rendering with billboard particles
- Orbital trails with fading
- Multiple presets: Solar System, Accretion Disk, Galaxy Collision
- Star field background
- Leapfrog integration for stability

**Run:**
```bash
# 2D version:
cargo run -p gravity_sim

# Full 3D version (recommended):
cargo run -p gravity_sim --bin gravity_sim_3d
```

**Controls:**
- `1/2/3` - Load presets (Solar System, Disk, Galaxy Collision)
- `Mouse drag` - Orbit camera (3D)
- `Scroll` - Zoom in/out
- `T` - Toggle trails
- `G` - Toggle grid
- `Space` - Pause/Resume
- `+/-` - Adjust time scale
- `R` - Reset view

---

### 3. Black Hole Simulation (`black_hole`)

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

### 4. Atoms Simulation (`atoms`)

Molecular dynamics simulation with atomic interactions.

**Features:**
- Coulomb forces for charged particles
- Lennard-Jones potential for van der Waals interactions
- Dynamic covalent bond formation
- Multiple molecule presets (H₂O, NaCl, CH₄)
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
cargo build -p solar_voyage --release
cargo build -p gravity_sim --release
cargo build -p black_hole --release
cargo build -p atoms --release
```

## Project Structure

```
.
├── Cargo.toml           # Workspace configuration
├── common/              # Shared graphics and utility code
│   └── src/
│       ├── lib.rs
│       ├── graphics.rs  # wgpu setup and vertex types
│       └── camera.rs    # 2D/3D camera systems
├── solar_voyage/        # Interstellar journey simulation
│   └── src/
│       ├── main.rs          # Main simulation
│       ├── solar_system.rs  # Accurate planetary data
│       ├── spaceship.rs     # Ship physics & relativity
│       ├── spacetime.rs     # Curvature visualization
│       └── renderer.rs      # Combined rendering
├── gravity_sim/         # N-body gravity simulation
│   └── src/
│       ├── main.rs          # 2D simulation
│       ├── main_3d.rs       # Full 3D simulation
│       ├── physics.rs       # 2D physics
│       ├── physics_3d.rs    # 3D physics with trails
│       ├── renderer.rs      # 2D rendering
│       └── renderer_3d.rs   # 3D rendering
├── black_hole/          # Black hole visualization
│   └── src/
│       ├── main.rs          # 3D ray-marching
│       ├── main_2d.rs       # 2D lensing visualization
│       ├── physics.rs       # Schwarzschild geodesics
│       └── renderer.rs      # Ray tracing renderer
└── atoms/               # Molecular dynamics
    └── src/
        ├── main.rs          # Main simulation
        ├── physics.rs       # Atomic forces and bonding
        └── renderer.rs      # Atom/bond rendering
```

## Physics Concepts

### Solar Voyage (Relativistic Physics)
- **Special Relativity**: Lorentz factor γ = 1/√(1 - v²/c²)
- **Time Dilation**: Proper time τ = t/γ (moving clocks run slower)
- **Gravitational Time Dilation**: τ = t√(1 - 2GM/rc²)
- **Relativistic Doppler Effect**: Blue/red shift based on direction of motion
- **Stellar Aberration**: Stars appear to bunch up in direction of travel

### Gravity Simulation
- **Newton's Law of Gravitation**: F = G × m₁ × m₂ / r²
- **Leapfrog Integration**: Symplectic integrator for orbital mechanics
- **Softening Parameter**: Prevents numerical instabilities at close approaches

### Black Hole Simulation
- **Schwarzschild Radius**: rs = 2GM/c² (event horizon)
- **Photon Sphere**: r = 1.5 × rs (unstable photon orbits)
- **Geodesic Equations**: Light paths through curved spacetime
- **Runge-Kutta 4 (RK4)**: Numerical integration of geodesics

### Atoms Simulation
- **Coulomb's Law**: F = k × q₁ × q₂ / r² (electrostatic forces)
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
