use avian3d::prelude::*;
use bevy::prelude::*;

const SAND: Color = Color::srgb(0.87, 0.72, 0.53);
const WATER: Color = Color::srgb(0.22, 0.62, 0.85);

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sand = materials.add(SAND);
    let water = materials.add(WATER);
    let m = &mut *meshes;

    // Ground
    commands.spawn((
        Mesh3d(m.add(Plane3d::default().mesh().size(200.0, 200.0))),
        MeshMaterial3d(sand.clone()),
        Transform::default(),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    spawn_walls(&mut commands, m, sand.clone());
    spawn_fountain(&mut commands, m, sand.clone(), water);
    spawn_market(&mut commands, m, sand.clone());
    spawn_east_terrace(&mut commands, m, sand.clone());
    spawn_west_terrace(&mut commands, m, sand.clone());
    spawn_castle(&mut commands, m, sand.clone());

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.5, 0.0)),
    ));
    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        ..default()
    });
}

// ── helpers ────────────────────────────────────────────────────────────────

fn box_at(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat: Handle<StandardMaterial>,
    pos: Vec3,
    size: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(mat),
        Transform::from_translation(pos),
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

fn cyl_at(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat: Handle<StandardMaterial>,
    pos: Vec3,
    radius: f32,
    height: f32,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cylinder {
            radius,
            half_height: height / 2.0,
        })),
        MeshMaterial3d(mat),
        Transform::from_translation(pos),
        RigidBody::Static,
        Collider::cylinder(radius, height ),
    ));
}

/// Ramp along the X axis. (x0,y0) is the low end, (x1,y1) the high end.
fn ramp_x(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat: Handle<StandardMaterial>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    z: f32,
    width: f32,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    let thick = 0.5;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(len, thick, width))),
        MeshMaterial3d(mat),
        Transform::from_xyz((x0 + x1) / 2.0, (y0 + y1) / 2.0, z)
            .with_rotation(Quat::from_rotation_z(dy.atan2(dx))),
        RigidBody::Static,
        Collider::cuboid(len , thick , width ),
    ));
}

// ── sections ───────────────────────────────────────────────────────────────

fn spawn_walls(commands: &mut Commands, meshes: &mut Assets<Mesh>, mat: Handle<StandardMaterial>) {
    for (pos, size) in [
        (Vec3::new(   0.0, 3.0, -101.0), Vec3::new(202.0, 6.0,   2.0)), // N
        (Vec3::new(   0.0, 3.0,  101.0), Vec3::new(202.0, 6.0,   2.0)), // S
        (Vec3::new( 101.0, 3.0,    0.0), Vec3::new(  2.0, 6.0, 202.0)), // E
        (Vec3::new(-101.0, 3.0,    0.0), Vec3::new(  2.0, 6.0, 202.0)), // W
    ] {
        box_at(commands, meshes, mat.clone(), pos, size);
    }
}

fn spawn_fountain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    sand: Handle<StandardMaterial>,
    water: Handle<StandardMaterial>,
) {
    cyl_at(commands, meshes, sand.clone(), Vec3::new(0.0, 0.5,  0.0), 2.5,  1.0); // base
    cyl_at(commands, meshes, sand.clone(), Vec3::new(0.0, 2.25, 0.0), 0.4,  2.5); // shaft
    cyl_at(commands, meshes, sand.clone(), Vec3::new(0.0, 3.75, 0.0), 2.0,  0.5); // basin rim
    cyl_at(commands, meshes, water,        Vec3::new(0.0, 3.85, 0.0), 1.6,  0.15); // water
}

fn spawn_market(commands: &mut Commands, meshes: &mut Assets<Mesh>, mat: Handle<StandardMaterial>) {
    // (x, z, w, h, d) — ground-level market buildings around the fountain
    for &(x, z, w, h, d) in &[
        // inner ring r≈13
        ( 13.0,   0.0, 5.0, 4.0, 5.0_f32),
        (-13.0,   0.0, 5.0, 4.0, 5.0),
        (  0.0,  13.0, 5.0, 4.0, 5.0),
        (  0.0, -14.0, 5.0, 5.0, 5.0),
        (  9.0,   9.0, 4.0, 3.5, 4.0),
        ( -9.0,   9.0, 4.0, 3.5, 4.0),
        (  9.0,  -9.0, 4.0, 5.0, 4.0),
        ( -9.0,  -9.0, 4.0, 5.0, 4.0),
        // outer ring r≈22
        ( 22.0,   0.0, 5.0, 3.5, 6.0),
        (-22.0,   0.0, 5.0, 3.5, 6.0),
        (  0.0,  22.0, 6.0, 3.5, 5.0),
        (  0.0, -23.0, 6.0, 4.0, 5.0),
        // alley stalls
        ( 17.0,   5.0, 1.5, 2.5, 8.0),
        (-17.0,  -5.0, 1.5, 2.5, 8.0),
    ] {
        box_at(
            commands,
            meshes,
            mat.clone(),
            Vec3::new(x, h / 2.0, z),
            Vec3::new(w, h, d),
        );
    }
}

fn spawn_east_terrace(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat: Handle<StandardMaterial>,
) {
    // Platform — top surface at y = 5.0, x: 44–80, z: −13–13
    box_at(
        commands, meshes, mat.clone(),
        Vec3::new(62.0, 4.5, 0.0),
        Vec3::new(36.0, 1.0, 26.0),
    );

    // Ramp from market level (x=30,y=0) up to platform edge (x=44,y=5)
    ramp_x(commands, meshes, mat.clone(), 30.0, 0.0, 44.0, 5.0, 0.0, 4.0);

    // Residential buildings — base_y = 5.0
    for &(x, z, w, h, d) in &[
        (50.0,  8.0, 6.0, 4.0, 5.0_f32),
        (58.0, -8.0, 5.0, 5.0, 5.0),
        (66.0,  0.0, 5.0, 4.0, 6.0),
        (72.0,  7.0, 4.0, 3.5, 4.0),
        (72.0, -7.0, 4.0, 3.5, 4.0),
        (50.0, -8.0, 5.0, 4.0, 5.0),
    ] {
        box_at(
            commands, meshes, mat.clone(),
            Vec3::new(x, 5.0 + h / 2.0, z),
            Vec3::new(w, h, d),
        );
    }

    // Parapet walls along terrace edges (h=1.2, base at y=5.0)
    for (pos, size) in [
        (Vec3::new( 62.0, 5.6,  13.0), Vec3::new(36.0, 1.2,  0.5)), // N
        (Vec3::new( 62.0, 5.6, -13.0), Vec3::new(36.0, 1.2,  0.5)), // S
        (Vec3::new( 80.0, 5.6,   0.0), Vec3::new( 0.5, 1.2, 26.0)), // E
    ] {
        box_at(commands, meshes, mat.clone(), pos, size);
    }
}

fn spawn_west_terrace(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat: Handle<StandardMaterial>,
) {
    box_at(
        commands, meshes, mat.clone(),
        Vec3::new(-62.0, 4.5, 0.0),
        Vec3::new(36.0, 1.0, 26.0),
    );

    ramp_x(commands, meshes, mat.clone(), -44.0, 5.0, -30.0, 0.0, 0.0, 4.0);

    for &(x, z, w, h, d) in &[
        (-50.0,  8.0, 6.0, 4.0, 5.0_f32),
        (-58.0, -8.0, 5.0, 5.0, 5.0),
        (-66.0,  0.0, 5.0, 4.0, 6.0),
        (-72.0,  7.0, 4.0, 3.5, 4.0),
        (-72.0, -7.0, 4.0, 3.5, 4.0),
        (-50.0, -8.0, 5.0, 4.0, 5.0),
    ] {
        box_at(
            commands, meshes, mat.clone(),
            Vec3::new(x, 5.0 + h / 2.0, z),
            Vec3::new(w, h, d),
        );
    }

    for (pos, size) in [
        (Vec3::new(-62.0, 5.6,  13.0), Vec3::new(36.0, 1.2,  0.5)),
        (Vec3::new(-62.0, 5.6, -13.0), Vec3::new(36.0, 1.2,  0.5)),
        (Vec3::new(-80.0, 5.6,   0.0), Vec3::new( 0.5, 1.2, 26.0)),
    ] {
        box_at(commands, meshes, mat.clone(), pos, size);
    }
}

fn spawn_castle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mat: Handle<StandardMaterial>,
) {
    let (cx, cz) = (0.0_f32, -72.0_f32);

    // Mound — four stacked cylinders tapering upward (MSM-style rocky hill)
    // Layer tops: y=6, 12, 18, 21
    cyl_at(commands, meshes, mat.clone(), Vec3::new(cx,  3.0, cz), 19.0, 6.0);
    cyl_at(commands, meshes, mat.clone(), Vec3::new(cx,  9.0, cz), 14.0, 6.0);
    cyl_at(commands, meshes, mat.clone(), Vec3::new(cx, 15.0, cz),  9.0, 6.0);
    cyl_at(commands, meshes, mat.clone(), Vec3::new(cx, 19.5, cz),  6.0, 3.0);

    // Platform on mound (top surface y = 21.6)
    box_at(
        commands, meshes, mat.clone(),
        Vec3::new(cx, 21.3, cz),
        Vec3::new(14.0, 0.6, 14.0),
    );

    // Main keep  (bottom y=21.6, top y=35.6)
    box_at(
        commands, meshes, mat.clone(),
        Vec3::new(cx, 28.6, cz),
        Vec3::new(10.0, 14.0, 10.0),
    );

    // Chapel wing — adjoins south face of keep (cz+5 = keep south edge)
    box_at(
        commands, meshes, mat.clone(),
        Vec3::new(cx, 25.6, cz + 9.0),
        Vec3::new(6.0, 8.0, 8.0),
    );

    // Apse (semicircular end of chapel)
    cyl_at(
        commands, meshes, mat.clone(),
        Vec3::new(cx, 24.6, cz + 14.0),
        3.0, 6.0,
    );

    // 4 corner towers — taller than the keep (tops at y≈43.6)
    for &(tx, tz) in &[
        ( 5.0_f32, -5.0_f32),
        (-5.0,     -5.0),
        ( 5.0,      5.0),
        (-5.0,      5.0),
    ] {
        cyl_at(
            commands, meshes, mat.clone(),
            Vec3::new(cx + tx, 32.6, cz + tz),
            1.5, 22.0,
        );
    }
}
