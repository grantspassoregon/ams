use crate::controls::{act, command};
use crate::state::{self, lens};
use crate::tab;
use aid::prelude::Clean;
use std::sync::Arc;
use winit::{event, event_loop, window};

/// Top level application state.
pub struct App {
    window: Arc<window::Window>,
    state: state::State,
    exit: bool,
}

impl App {
    pub async fn boot() -> Clean<(Self, event_loop::EventLoop<()>)> {
        let icon = state::State::load_icon(include_bytes!("../data/gp_logo.png"))?;
        let event_loop = event_loop::EventLoop::new()?;
        let window = window::WindowBuilder::new()
            .with_title("AMS")
            .with_window_icon(Some(icon))
            .build(&event_loop)?;
        let window = Arc::new(window);
        let mut state = state::State::new(Arc::clone(&window)).await;
        if let Ok(lens) = lens::Lens::load("data/state.data") {
            state.lens = lens.clone();
            state.tab = tab::TabState::new(lens.clone());
            // state.tab = egui_dock::DockState::new(vec![tab::Tab::new(lens)]);
        } else {
            tracing::info!("Could not read state from storage.");
        }

        Ok((
            Self {
                window,
                state,
                exit: false,
            },
            event_loop,
        ))
    }

    pub async fn run(mut self, event_loop: event_loop::EventLoop<()>) -> Clean<()> {
        let _ = event_loop.run(move |event, ewlt| {
            ewlt.set_control_flow(event_loop::ControlFlow::Wait);
            if self.exit {
                ewlt.exit()
            }

            match event {
                event::Event::AboutToWait => {
                    self.state.about_to_wait();
                }
                event::Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.state.window.id() => {
                    match event {
                        event::WindowEvent::CloseRequested => {
                            self.close_requested();
                        }
                        event::WindowEvent::ModifiersChanged(modifiers) => {
                            self.state.modifiers = modifiers.state();
                            tracing::trace!("Modifiers changed to {:?}", self.state.modifiers);
                        }
                        event::WindowEvent::KeyboardInput {
                            event,
                            is_synthetic: false,
                            ..
                        } => {
                            self.keyboard_input(event);
                        }
                        event::WindowEvent::Resized(physical_size) => {
                            self.state.resize(*physical_size);
                        }
                        event::WindowEvent::RedrawRequested => match self.state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                self.state.resize(self.state.size)
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => self.exit = true,
                            Err(wgpu::SurfaceError::Timeout) => {
                                // Ignore timeouts.
                            }
                        },
                        other => {
                            self.state.handle_event(other);
                            self.window.request_redraw();
                            return;
                        }
                    };
                    self.state.handle_event(event);
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
        Ok(())
    }

    pub fn keyboard_input(&mut self, event: &event::KeyEvent) {
        // Dispatch actions only on press.
        if event.state.is_pressed() {
            // Interpret command.
            let command = match event.logical_key.as_ref() {
                winit::keyboard::Key::Named(k) => Some(command::Command::from(&k)),
                winit::keyboard::Key::Character(k) => {
                    Some(command::Command::new(&k, &self.state.modifiers))
                }
                _ => None,
            };

            // If command is valid
            if let Some(command) = command {
                tracing::trace!("{:#?}", &command);
                // Clone the command map
                let choices = self.state.command.clone();
                // Look up the current set of choices using the command key
                if let Some(choices) = choices.choices().0.get(&self.state.command_key) {
                    // Look up the command options given the current command
                    if let Some(opts) = choices.0.get(&command) {
                        match opts {
                            // If a command group, set the command key to the id of the group
                            command::CommandOptions::Commands(c) => {
                                tracing::trace!("Commands available: {:#?}", c);
                                self.state.command_key = c.id.clone();
                            }
                            // Take action
                            command::CommandOptions::Acts(a) => {
                                self.act(a);
                            }
                        }
                    } else {
                        tracing::trace!("Command not recognized.");
                    }
                }
            };
        }
    }

    pub fn act(&mut self, acts: &Vec<act::Act>) {
        tracing::trace!("Acts in queue: {:#?}", acts);
        // If an act, reset the command key to normal
        self.state.command_key = "normal".to_string();
        // for each act in queue
        for act in acts {
            match act {
                // dispatch to the appropriate handler
                act::Act::App(v) => self.state.act(v),
                act::Act::Egui(v) => self.state.tab.act(v),
                act::Act::Named(v) => {
                    tracing::trace!("{:#?}", &v);
                    match v {
                        act::NamedAct::Escape => {
                            self.close_requested();
                        }
                        act::NamedAct::Enter => self.state.lens.focus_tree.enter(),
                        _ => tracing::trace!("Named event detected"),
                    }
                }
                act::Act::Be => {
                    tracing::trace!("Taking no action.")
                }
            }
        }
    }

    pub fn close_requested(&mut self) {
        tracing::info!("Close requested.");
        let state = self.state();
        if state.lens.save("data/state.data").is_ok() {
            tracing::info!("State saved from ref.");
        } else {
            tracing::info!("Unable to save state to file.");
        }
        self.exit = true;
    }

    pub fn state(&self) -> &state::State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut state::State {
        &mut self.state
    }

    pub fn set_exit(set: &mut bool) {
        *set = true;
    }
}
