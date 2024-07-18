use crate::controls::{act, command};
use crate::prelude::{
    Action, EguiState, GalileoState, MatchPoints, UiState, WgpuFrame, KEY_BINDINGS, MOUSE_BINDINGS,
};
use crate::state::lens;
use crate::tab;
use aid::prelude::Clean;
use std::{iter, sync::Arc};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::*;
use winit::event_loop::EventLoop;
use winit::keyboard::ModifiersState;
use winit::window::{Fullscreen, Icon, Theme, Window, WindowId};

pub struct State {
    pub surface: Arc<wgpu::Surface<'static>>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub window: Arc<Window>,
    pub egui_state: EguiState,
    pub lens: lens::Lens,
    pub tab: tab::TabState,
    pub ui_state: UiState,
    pub galileo_state: GalileoState,
    pub modifiers: ModifiersState,
    pub theme: Theme,
    /// Cursor position over the window.
    pub cursor_position: Option<PhysicalPosition<f64>>,
    pub command: command::CommandMode,
    pub command_key: String,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits {
                            // NOTE(alexkirsz) These are the limits on my GPU w/ WebGPU,
                            // but your mileage may vary.
                            max_texture_dimension_2d: 16384,
                            ..wgpu::Limits::downlevel_webgl2_defaults()
                        }
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let egui_state = EguiState::new(&device, config.format, None, 1, &window);

        let surface = Arc::new(surface);
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let galileo_state = GalileoState::new(
            Arc::clone(&window),
            Arc::clone(&device),
            Arc::clone(&surface),
            Arc::clone(&queue),
            config.clone(),
        );

        let theme = window.theme().unwrap_or(Theme::Dark);
        let command = command::CommandMode::new();
        tracing::trace!("Commands: {:#?}", &command);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            egui_state,
            lens: lens::Lens::new(),
            tab: tab::TabState::default(),
            ui_state: UiState::new(),
            galileo_state,
            modifiers: Default::default(),
            theme,
            cursor_position: Default::default(),
            command,
            command_key: "normal".to_string(),
        }
    }

    pub fn about_to_wait(&mut self) {
        // tracing::info!("Removed call to galileo_state.");
        self.galileo_state.about_to_wait();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.galileo_state.resize(new_size);
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        let res = self.egui_state.handle_event(&self.window, event);
        if !res.consumed {
            self.galileo_state.handle_event(event);
        }

        if let Some(table) = &mut self.ui_state.operations.compare.table {
            if let Some(package) = table.package.take() {
                tracing::trace!("Package taken.");
                let points = MatchPoints::from(&package);
                self.galileo_state.addresses = Some(points);
                self.galileo_state.load_addresses(1).unwrap();
                tracing::trace!("Records added to map.");
            }
        }

        // Only load lexis nexis data if the lexis window is open
        if self.ui_state.operations.lexis_visible() {
            // Copy boundary layer to galileo
            if let Some(pkg) = &self.ui_state.operations.lexis.boundary_pkg.take() {
                // Move layer to galileo_state
                self.galileo_state.boundary = Some(pkg.clone());
                // Load layer into display.
                self.galileo_state.load_boundary(1).unwrap();
            }

            // Load address results to galileo
            if let Some(view) = &self.ui_state.operations.lexis.address_pkg.take() {
                // Move layer data to galileo_state
                self.galileo_state.lexis = Some(view.clone());
                // Load layer into display.
                self.galileo_state.load_lexis(2).unwrap();
            }
        }

        self.window.request_redraw();
    }

    pub fn load_icon(bytes: &[u8]) -> Clean<Icon> {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::load_from_memory(bytes)
                .expect("Could not load icon.")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        Ok(Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Bad icon."))
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let texture = self.surface.get_current_texture()?;

        let texture_view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut wgpu_frame = WgpuFrame {
                device: &self.device,
                queue: &self.queue,
                encoder: &mut encoder,
                window: &self.window,
                texture_view: &texture_view,
                size: self.size,
            };

            self.galileo_state.render(&wgpu_frame);

            self.egui_state
                // .render(&mut wgpu_frame, |ui| self.ui_state.run(ui));
                .render(&mut wgpu_frame, |ui| self.tab.run_ui(ui));
        }

        self.queue.submit(iter::once(encoder.finish()));

        texture.present();

        Ok(())
    }
    /// Process the key binding.
    pub fn process_key_binding(key: &str, mods: &ModifiersState) -> Option<Action> {
        KEY_BINDINGS.iter().find_map(|binding| {
            binding
                .is_triggered_by(&key, mods)
                .then_some(binding.action)
        })
    }

    /// Process mouse binding.
    pub fn process_mouse_binding(button: MouseButton, mods: &ModifiersState) -> Option<Action> {
        MOUSE_BINDINGS.iter().find_map(|binding| {
            binding
                .is_triggered_by(&button, mods)
                .then_some(binding.action)
        })
    }
    pub fn print_help(&self) {
        tracing::trace!("Keyboard bindings:");
        for binding in KEY_BINDINGS {
            tracing::trace!(
                "{:?}{:<10} - {} ({})",
                binding.mods,
                binding.trigger,
                binding.action,
                binding.action.help(),
            );
        }
        tracing::trace!("Mouse bindings:");
        for binding in MOUSE_BINDINGS {
            tracing::trace!(
                "{:?}{:#?} - {} ({})",
                binding.mods,
                binding.trigger,
                binding.action,
                binding.action.help(),
            );
        }
    }

    /// Minimize the window.
    pub fn minimize(&mut self) {
        self.window.set_minimized(true);
    }

    /// Change the theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.window.request_redraw();
    }

    /// Show window menu.
    pub fn show_menu(&self) {
        if let Some(position) = self.cursor_position {
            self.window.show_window_menu(position);
        }
    }

    /// Toggle window decorations.
    pub fn toggle_decorations(&self) {
        let decorated = self.window.is_decorated();
        self.window.set_decorations(!decorated);
    }

    /// Toggle fullscreen.
    pub fn toggle_fullscreen(&self) {
        let fullscreen = if self.window.fullscreen().is_some() {
            None
        } else {
            Some(Fullscreen::Borderless(None))
        };

        self.window.set_fullscreen(fullscreen);
    }

    /// Toggle maximized.
    pub fn toggle_maximize(&self) {
        let maximized = self.window.is_maximized();
        self.window.set_maximized(!maximized);
    }

    pub fn handle_action(
        &mut self,
        _event_loop: &EventLoop<()>,
        _window_id: WindowId,
        action: Action,
    ) {
        //     // let cursor_position = self.cursor_position;
        //     // let window = self.windows.get_mut(&window_id).unwrap();
        //     println!("Executing action: {action:?}");
        match action {
            //         Action::CloseWindow => {
            //             // let _ = self.window.remove(&window_id);
            //         }
            //         // Action::CreateNewWindow => {
            //         //     #[cfg(any(x11_platform, wayland_platform))]
            //         //     if let Err(err) = window.window.request_activation_token() {
            //         //         println!("Failed to get activation token: {err}");
            //         //     } else {
            //         //         return;
            //         //     }
            //         //
            //         //     if let Err(err) = self.create_window(event_loop, None) {
            //         //         eprintln!("Error creating new window: {err}");
            //         //     }
            //         // }
            //         Action::ToggleResizeIncrements => self.toggle_resize_increments(),
            //         Action::ToggleCursorVisibility => window.toggle_cursor_visibility(),
            //         Action::ToggleResizable => window.toggle_resizable(),
            //         Action::ToggleDecorations => window.toggle_decorations(),
            //         Action::ToggleFullscreen => window.toggle_fullscreen(),
            //         Action::ToggleMaximize => window.toggle_maximize(),
            //         Action::ToggleImeInput => window.toggle_ime(),
            //         Action::Minimize => window.minimize(),
            //         Action::NextCursor => window.next_cursor(),
            //         Action::NextCustomCursor => window.next_custom_cursor(&self.custom_cursors),
            //         Action::CycleCursorGrab => window.cycle_cursor_grab(),
            //         Action::DragWindow => window.drag_window(),
            //         Action::DragResizeWindow => window.drag_resize_window(),
            //         Action::ShowWindowMenu => window.show_menu(),
            Action::PrintHelp => self.print_help(),
            //         #[cfg(macos_platform)]
            //         Action::CycleOptionAsAlt => window.cycle_option_as_alt(),
            //         #[cfg(macos_platform)]
            //         Action::CreateNewTab => {
            //             let tab_id = window.window.tabbing_identifier();
            //             if let Err(err) = self.create_window(event_loop, Some(tab_id)) {
            //                 eprintln!("Error creating new window: {err}");
            //             }
            _ => tracing::trace!("Other action!"),
        }
        //     }
    }

    pub fn act(&mut self, act: &act::AppAct) {
        match *act {
            act::AppAct::Help => {}
            act::AppAct::Menu => self.show_menu(),
            act::AppAct::Decorations => self.toggle_decorations(),
            act::AppAct::Fullscreen => self.toggle_fullscreen(),
            act::AppAct::Maximize => self.toggle_maximize(),
            act::AppAct::Minimize => self.minimize(),
            act::AppAct::Be => tracing::trace!("No action taken."),
        }
    }
}
