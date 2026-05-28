use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use neotui_core::component::LayoutContext;
use neotui_core::dsl::AppSpec;
use neotui_core::layout::Rect;
use neotui_core::registry::ComponentRegistry;
use neotui_core::render::{FrameDiff, ScreenBuffer};

const BENCH_ITERATIONS: usize = 1_000;

struct BenchmarkReport {
    name: &'static str,
    iterations: usize,
    elapsed: Duration,
}

impl BenchmarkReport {
    fn average_micros(&self) -> f64 {
        self.elapsed.as_secs_f64() * 1_000_000.0 / self.iterations as f64
    }

    fn emit(&self) {
        println!(
            "benchmark {}: iterations={} total_ms={:.3} avg_us={:.3}",
            self.name,
            self.iterations,
            self.elapsed.as_secs_f64() * 1_000.0,
            self.average_micros()
        );
    }
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join(name)
}

fn load_tree(name: &str) -> neotui_core::component::ComponentTree {
    let input = fs::read_to_string(fixture_path(name)).expect("fixture should exist");
    let spec = AppSpec::from_toml_str(&input).expect("fixture should parse");
    ComponentRegistry::new()
        .build_tree(&spec)
        .expect("fixture should instantiate")
}

fn benchmark(name: &'static str, iterations: usize, mut run: impl FnMut()) -> BenchmarkReport {
    let started = Instant::now();
    for _ in 0..iterations {
        run();
    }

    BenchmarkReport {
        name,
        iterations,
        elapsed: started.elapsed(),
    }
}

#[test]
#[ignore = "manual benchmark"]
fn benchmark_dashboard_layout() {
    let tree = load_tree("dashboard.toml");
    let area = Rect::new(0, 0, 120, 40);
    let report = benchmark("dashboard_layout", BENCH_ITERATIONS, || {
        let layout = tree.layout(&LayoutContext, area.clone());
        black_box(layout);
    });

    report.emit();
}

#[test]
#[ignore = "manual benchmark"]
fn benchmark_showcase_render() {
    let tree = load_tree("showcase-layout.toml");
    let area = Rect::new(0, 0, 120, 40);
    let layout = tree.layout(&LayoutContext, area);
    let mut frame = ScreenBuffer::new(120, 40);
    let report = benchmark("showcase_render", BENCH_ITERATIONS, || {
        frame.clear();
        tree.render_with_layout(&layout, &mut frame);
        black_box(frame.cells());
    });

    report.emit();
}

#[test]
#[ignore = "manual benchmark"]
fn benchmark_frame_diff() {
    let tree = load_tree("dashboard.toml");
    let area = Rect::new(0, 0, 120, 40);
    let layout = tree.layout(&LayoutContext, area);
    let mut previous = ScreenBuffer::new(120, 40);
    let mut current = ScreenBuffer::new(120, 40);

    tree.render_with_layout(&layout, &mut previous);
    tree.render_with_layout(&layout, &mut current);
    let _ = current.draw_text(0, 0, "benchmark-delta", Default::default());

    let report = benchmark("frame_diff", BENCH_ITERATIONS, || {
        let diff = FrameDiff::between(&previous, &current);
        black_box(diff);
    });

    report.emit();
}
