use cellworld::{CellType, Coord3, World};

#[test]
fn cells_consume_feed_at_their_position() {
    let mut world = World::new();
    let position = Coord3::new(0, 0, 0);
    world.add_feed(position, 2.0);
    world.add_cell(position);

    world.step();

    let cell = world.cells.first().expect("cell should survive");
    assert!(cell.energy > 0.0);
    assert!(world.feed.get(&position).copied().unwrap_or_default() < 2.0);
}

#[test]
fn nearby_cells_raise_local_signal() {
    let mut world = World::new();
    let a = Coord3::new(0, 0, 0);
    let b = Coord3::new(1, 0, 0);
    world.add_cell(a);
    world.add_cell(b);

    world.step();

    let first = world.cells.iter().find(|cell| cell.position == a).unwrap();
    let second = world.cells.iter().find(|cell| cell.position == b).unwrap();

    assert!(first.fast_signal > 0.0);
    assert!(second.fast_signal > 0.0);
}

#[test]
fn cells_die_without_feed() {
    let mut world = World::new();
    let position = Coord3::new(4, 4, 4);
    world.add_cell(position);

    for _ in 0..5 {
        world.step();
    }

    assert_eq!(world.cell_count(), 0);
}

#[test]
fn an_empty_world_stays_empty_when_stepped() {
    let mut world = World::new();

    for _ in 0..5 {
        world.step_with_feed(1.5);
    }

    assert_eq!(world.cell_count(), 0);
    assert!(world.feed.values().next().is_none());
}

#[test]
fn cells_move_towards_available_feed() {
    let mut world = World::new();
    world.add_cell(Coord3::new(0, 0, 0));
    world.add_feed(Coord3::new(1, 0, 0), 3.0);

    world.step();

    let cell = world.cells.first().expect("cell should still exist");
    assert_eq!(cell.position, Coord3::new(1, 0, 0));
}

#[test]
fn cells_reproduce_when_energy_is_high() {
    let mut world = World::new();
    world.add_feed(Coord3::new(0, 0, 0), 20.0);
    world.add_cell(Coord3::new(0, 0, 0));

    for _ in 0..12 {
        world.add_feed(Coord3::new(0, 0, 0), 20.0);
        world.step();
        if world.cell_count() > 1 {
            break;
        }
    }

    assert!(
        world.cell_count() >= 2,
        "fed cells should reproduce once they have enough energy"
    );
}

#[test]
fn feed_can_be_generated_per_tick() {
    let mut world = World::new();
    world.add_cell(Coord3::new(0, 0, 0));

    world.step_with_feed(2.5);

    let cell = world.cells.first().expect("cell should exist");
    assert!(world.feed.get(&cell.position).copied().unwrap_or_default() > 0.0);
}

#[derive(Clone, Debug)]
struct EfficientCell;

impl CellType for EfficientCell {
    fn name(&self) -> &'static str {
        "Efficient"
    }

    fn energy_cost(&self) -> f32 {
        0.1
    }

    fn clone_type(&self) -> Box<dyn CellType> {
        Box::new(self.clone())
    }
}

#[test]
fn custom_cell_types_supply_their_own_simulation_rules() {
    let mut world = World::new();
    let position = Coord3::new(0, 0, 0);
    world.add_cell(position);
    world.add_cell_with_type(position, EfficientCell);

    world.step();

    let basic = world
        .cells
        .iter()
        .find(|cell| cell.cell_type().name() == "Basic")
        .unwrap();
    let efficient = world
        .cells
        .iter()
        .find(|cell| cell.cell_type().name() == "Efficient")
        .unwrap();
    assert!(efficient.energy > basic.energy);
}

#[test]
fn cloning_world_preserves_cell_type_identity() {
    let mut world = World::new();
    world.add_cell(Coord3::new(0, 0, 0));
    world.add_cell_with_type(Coord3::new(1, 0, 0), EfficientCell);

    let clone = world.clone();

    assert_eq!(world, clone);
    assert_ne!(
        world.cells[0].cell_type().name(),
        world.cells[1].cell_type().name()
    );
}

#[test]
fn offspring_inherit_the_parent_cell_type() {
    let mut world = World::new();
    let position = Coord3::new(0, 0, 0);
    world.add_feed(position, 20.0);
    world.add_cell_with_type(position, EfficientCell);

    for _ in 0..12 {
        world.add_feed(position, 20.0);
        world.step();
        if world.cell_count() > 1 {
            break;
        }
    }

    assert!(
        world.cells.len() > 1,
        "the fed custom cell should reproduce"
    );
    assert!(
        world
            .cells
            .iter()
            .all(|cell| cell.cell_type().name() == "Efficient")
    );
}
