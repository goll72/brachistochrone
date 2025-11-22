use nalgebra::Vector2;

mod brachistochrone;

use brachistochrone::Brachistochrone;

fn main() {
    let start = Vector2::<f32>::new(0., 100.);
    let end = Vector2::<f32>::new(100., 0.);

    let mut b = Brachistochrone::new(100, 50, 0.1, start, end);

    b.solve();

    for (t, r) in b.path_iter(start) {
        println!(
            "A brachistochrone-like path would take {t:0.5} seconds to get to {end:?} starting from {:?}, {:?}",
            r.x, r.y
        );
    }
}
