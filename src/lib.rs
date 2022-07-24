#[macro_export]
macro_rules! register_rhai_type {
    ($name:literal = $ty:ty => $engine:expr;
        $($func:ident),* $(,)*
    ) => {
        $engine
            .register_type_with_name::<$ty>($name)
            $(
                .register_fn(stringify!($func), <$ty>::$func)
            )*
    };

    ($ty:ty => $engine:expr;
        $($func:ident),* $(,)*
    ) => {
        $engine
            .register_type::<$ty>()
            $(
                .register_fn(stringify!($func), <$ty>::$func)
            )*
    };
}


pub mod modules;

use std::{mem, sync::Arc};

use eframe::egui::{self, Context};
use parking_lot::Mutex;
use rhai::{exported_module, Engine, ParseError, Scope};

#[derive(Clone)]
pub struct RyeUi {
    ui: *mut egui::Ui,
}

impl RyeUi {
    fn new(ui: *mut egui::Ui) -> Self {
        Self { ui }
    }

    fn heading(&mut self, text: &str) -> egui::Response {
        self.ui().heading(text)
    }

    fn ui(&mut self) -> &mut egui::Ui {
        unsafe { &mut *self.ui }
    }
}

pub struct Rye {
    engine: Engine,
    script: Arc<Mutex<Option<String>>>,
    ast: rhai::AST,

    on_change: Option<Box<dyn Fn()>>,
    on_update: Option<Box<dyn FnMut(&mut Scope)>>,
}

impl Default for Rye {
    fn default() -> Self {
        let mut s = Self {
            engine: Default::default(),
            script: Default::default(),
            ast: Default::default(),
            on_change: Default::default(),
            on_update: Default::default(),
        };
        s.setup_engine();
        s
    }
}

impl Rye {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile_ast(&mut self) {
        if let Some(script) = self.script.lock().take() {
            match self.engine.compile(&script) {
                Ok(ast) => self.ast = ast,
                Err(err) => {
                    println!("{}", err);
                }
            }
            if let Some(cb) = &self.on_change {
                (*cb)();
            }
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        let mut scope = Scope::new();

        self.update_with_scope(ctx, &mut scope)
    }

    pub fn update_with_scope(&mut self, ctx: &Context, scope: &mut Scope) {
        scope.set_or_push("egui_ctx", ctx.clone());

        self.compile_ast();
        if let Err(err) = self.engine.run_ast_with_scope(scope, &self.ast) {
            println!("error: {}", err);
        }
    }

    pub fn on_change(&mut self, handler: impl Fn() + 'static) {
        self.on_change = Some(Box::new(handler));
    }

    pub fn on_update(&mut self, handler: impl FnMut(&mut Scope) + 'static) {
        self.on_update = Some(Box::new(handler));
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn replace_engine(&mut self, engine: Engine) -> Engine {
        let eng = mem::replace(&mut self.engine, engine);
        self.setup_engine();

        eng
    }

    pub fn set_engine(&mut self, engine: Engine) {
        self.engine = engine;
        self.setup_engine();
    }

    pub fn handle(&self) -> Handle {
        Handle {
            script: self.script.clone(),
        }
    }

    fn setup_engine(&mut self) {
        self.engine.set_max_expr_depths(0, 0);

        self.engine
            .register_type_with_name::<egui::Response>("UiResponse")
            .register_type_with_name::<egui::Context>("UiContext")
            .register_type_with_name::<RyeUi>("Ui")
            .register_fn("heading", RyeUi::heading);

        self.engine
            .register_static_module("egui", exported_module!(modules::egui).into());


        register_rhai_type!(
            "Ui" = RyeUi => self.engine;
            heading
        );
    }
}

#[derive(Clone)]
pub struct Handle {
    script: Arc<Mutex<Option<String>>>,
}

impl Handle {
    pub fn set_script(&self, script: &str) -> Result<(), ParseError> {
        *self.script.lock() = Some(script.into());
        Ok(())
    }
}
