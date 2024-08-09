use pollster::FutureExt;
use rpenguin::run;

fn main() {
    // pollster::block_on(run());
    run().block_on();
}
