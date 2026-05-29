# Camera System

## OrbitCamera Component

`apps/desktop/src/camera.rs:7` — A Bevy `Component` attached to the 3D camera entity (spawned in `setup()` at `main.rs:125`).

```rust
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub fov_y: f32,
    pub sensitivity: f32,
    pub zoom_speed: f32,
}
```

- **target**: The 3D point the camera orbits around. Default: `Vec3::ZERO`.
- **yaw / pitch**: Spherical coordinate angles (in radians). Yaw orbits horizontally; pitch orbits vertically. Pitch is clamped to ±1.5 rad (~±86°) to prevent flipping.
- **distance**: Radial distance from target to eye. Clamped to [0.5, 100.0].
- **fov_y**: Vertical field of view (default 45° in radians).
- **sensitivity**: Mouse movement scaling for orbit/pan (default 0.005).
- **zoom_speed**: Scroll wheel scaling factor (default 0.1 — each scroll tick multiplies distance by 0.9 or 1.1).

## orbit_camera_system

`camera.rs:32` — Runs every frame in the `Update` schedule. It handles four interactions:

### Right Mouse Button — Orbit

`camera.rs:56-59` — When RMB is held, mouse delta is applied to yaw and pitch:
```rust
camera.yaw -= delta.x * camera.sensitivity;
camera.pitch += delta.y * camera.sensitivity;
camera.pitch = camera.pitch.clamp(-1.5, 1.5);
```

### Middle Mouse Button — Pan

`camera.rs:63-69` — When MMB is held, the camera target is translated in the camera's local right/up plane. Pan speed scales with distance so that close-up panning is proportionally smaller.

```rust
let pan_speed = camera.distance * 0.002;
camera.target -= right * delta.x * pan_speed;
camera.target += up * delta.y * pan_speed;
```

### Scroll — Zoom

`camera.rs:72-74` — Scroll Y moves the camera along the radial axis:
```
distance *= 1.0 - scroll * zoom_speed
```

### Keyboard — View Presets

`camera.rs:82-106` — Number keys snap the camera to standard viewpoints:

| Key | View | Yaw | Pitch | Distance |
|---|---|---|---|---|
| `1` | Front | 0.0 | 0.0 | 12.0 |
| `2` | Top | 0.0 | ~89.9° (π/2 - 0.01) | 12.0 |
| `3` | Right | 90° (π/2) | 0.0 | 12.0 |
| `4` | Isometric | ~23° (0.4 rad) | ~23° (0.4 rad) | 10.0 |

### F — Fit All

`camera.rs:78-80` — Calls `fit_all()` which computes the axis-aligned bounding box of all entities with `MeshNodeId` and `Aabb` components. The camera target is set to the AABB center, and distance is set to `max(radius + 2.0, 2.0)` where `radius = half-diagonal of the bounding box`. Yaw and pitch reset to isometric values (0.4, 0.4).

## Camera Position from Spherical Coordinates

`camera.rs:108-121` — At the end of each frame, the camera position is computed from the spherical coordinate parameters:

```rust
let eye = camera.target + Vec3::new(
    camera.distance * pitch_cos * yaw_sin,
    camera.distance * pitch_sin,
    camera.distance * pitch_cos * yaw_cos,
);
transform.translation = eye;
transform.look_at(camera.target, Vec3::Y);
```

This is a standard spherical-to-Cartesian conversion where:
- Y is the up axis
- Yaw rotates around Y (longitude)
- Pitch rotates from the horizontal plane (latitude)

`look_at()` orients the camera to face `target` with `Vec3::Y` as the up direction.
