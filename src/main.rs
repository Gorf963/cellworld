use std::env;

use cellworld::{Coord3, World};

fn main() {
    let feed_per_tick = env::args()
        .nth(1)
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(1.5);

    let mut world = World::new();
    world.add_feed(Coord3::new(0, 0, 0), 2.8);
    world.add_cell(Coord3::new(0, 0, 0));
    world.add_cell(Coord3::new(1, 0, 0));
    world.add_cell(Coord3::new(-1, 0, 0));

    for tick in 1..=5 {
        world.step_with_feed(feed_per_tick);
        println!(
            "tick {tick}: cells={}, total_energy={:.2}, feed={:.2}",
            world.cell_count(),
            world.total_energy(),
            world.feed.values().sum::<f32>()
        );
    }
}
