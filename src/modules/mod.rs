use rhai::plugin::*;

#[export_module]
pub(crate) mod egui {
    pub mod layout {
        use eframe::egui::CentralPanel;
        use rhai::Dynamic;

        use eframe::egui::Context;
        use rhai::FnPtr;

        use crate::RyeUi;

        pub fn central_panel(context: NativeCallContext, ctx: Context, cb: FnPtr) {
            CentralPanel::default().show(&ctx, |ui| {
                cb.call_dynamic(&context, None, [Dynamic::from(RyeUi::new(ui))])
            });
        }
    }
}
