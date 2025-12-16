use linearize::Linearize;

mod core {}

#[derive(Linearize)]
enum _X {
    A,
}

#[derive(Linearize)]
enum _Y {}
