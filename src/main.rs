use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};

use eframe::{egui, epi};
use gui::Rye;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rhai::{Engine, Scope};

#[derive(Default, Clone)]
struct Data {
    frame_time_ns: Arc<AtomicU64>,
}

impl Data {
    fn frame_time_ns(&mut self) -> u64 {
        self.frame_time_ns.load(Ordering::Acquire)
    }
}

struct TestApp {
    _watcher: RecommendedWatcher,
    data: Data,
    rye: Rye,
}

impl TestApp {
    fn new(script_path: impl AsRef<Path>) -> Self {
        let mut engine = Engine::default();

        engine
            .register_type::<Data>()
            .register_fn("frame_time_ns", Data::frame_time_ns);

        let mut rye = Rye::new();
        rye.set_engine(engine);

        let data = Data::default();

        let pb = PathBuf::from(script_path.as_ref());

        let s = fs::read_to_string(&pb).unwrap();
        let handle = rye.handle();
        let _ = handle.set_script(&s);

        let mut watcher = notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| match res {
                Ok(_event) => {
                    let s = fs::read_to_string(&pb).unwrap();
                    let _ = handle.set_script(&s);
                }
                Err(e) => println!("watch error: {:?}", e),
            },
        )
        .unwrap();

        watcher
            .watch(script_path.as_ref(), RecursiveMode::Recursive)
            .unwrap();

        Self {
            _watcher: watcher,
            data,
            rye,
        }
    }
}

impl epi::App for TestApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        frame.request_repaint();
        let start = Instant::now();
    
        let mut scope = Scope::new();

        scope.push("data", self.data.clone());

        self.rye.update_with_scope(ctx, &mut scope);

        let time = Instant::now() - start;
        self.data
            .frame_time_ns
            .store(time.as_nanos() as _, Ordering::Release);
    }

    fn warm_up_enabled(&self) -> bool {
        true
    }
}

fn main() {
    let app = TestApp::new("./gui.rhai");
    eframe::run_native(Box::new(app), eframe::NativeOptions::default());
}
