use pollster::FutureExt;
use wgpu_testing::run;

fn main() {
    // pollster::block_on(run());
    run().block_on();
}
