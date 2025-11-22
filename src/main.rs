use std::fs::File;
use std::io::Write;
use std::time::SystemTime;

use nalgebra::Vector2;

mod brachistochrone;

use brachistochrone::Brachistochrone;

fn main() {
    let ks = [
        (10, 3.),
        (20, 5.),
        (50, 7.),
        (100, 15.),
        (150, 30.),
        (200, 40.),
        (500, 50.),
    ];

    for (k, inc) in ks {
        let mut start = Vector2::<f32>::new(0., k as f32);
        let end = Vector2::<f32>::new(k as f32, 0.);

        let mut b = Brachistochrone::new(k, 10. / (k as f32), start, end);

        println!("Solving Brachistochrone for n={k}");

        let before = SystemTime::now();
        b.solve();

        println!(" >> Took {}s", before.elapsed().unwrap().as_secs_f64());

        while start.x < k as f32 / 2. {
            let path = format!("brac-{}-x{}.csv", k, start.x);
            println!(
                " :: Iterating path starting from [{:2.2}, {:2.2}] -> {path}",
                start.x, start.y
            );

            let mut f = File::options().write(true).create(true).open(path).unwrap();

            for (t, r) in b.path_iter(start) {
                write!(f, "{},{},{}\n", r.x, r.y, t).unwrap();
            }

            write!(f, "{},{},{}\n", end.x, end.y, 0).unwrap();

            start.x += inc;
        }
    }
}
