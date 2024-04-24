use crate::prelude::{Action, App};
use std::sync::Arc;
use winit::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};
use wgpu::SurfaceError;

pub async fn run(window: Window, event_loop: EventLoop<()>) {
    let window = Arc::new(window);

    let mut state = App::new(Arc::clone(&window)).await;

    let _ = event_loop.run(move |event, ewlt| {
        ewlt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::AboutToWait => {
                state.about_to_wait();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key:
                                    Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => ewlt.exit(),
                    WindowEvent::ModifiersChanged(modifiers) => {
                        state.modifiers = modifiers.state();
                        tracing::info!("Modifiers changed to {:?}", state.modifiers);
            }
                    WindowEvent::KeyboardInput {
                        event,
                        is_synthetic: false,
                        ..
                    } => {
                        let mods = state.modifiers;

                        // Dispatch actions only on press.
                        if event.state.is_pressed() {
                            tracing::info!("{:#?}", &event);
                            let action = if let Key::Character(ch) = event.logical_key.as_ref() {
                                App::process_key_binding(&ch.to_uppercase(), &mods)
                            } else {
                                None
                            };

                            if let Some(action) = action {
                                tracing::info!("{:#?}", &action);
                                match action {
                                    Action::Minimize => state.minimize(),
                                    Action::PrintHelp => state.print_help(),
                                    Action::ShowWindowMenu => state.show_menu(),
                                    Action::ToggleDecorations => state.toggle_decorations(),
                                    Action::ToggleFullscreen => state.toggle_fullscreen(),
                                    Action::ToggleMaximize => state.toggle_maximize(),
                                    _ => tracing::info!("Other action."),
                                }
                                // state.handle_action(&event_loop, window_id, action);
                            }
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => match state.render() {
                        Ok(_) => {}
                        Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                            state.resize(state.size)
                        }
                        Err(SurfaceError::OutOfMemory) => ewlt.exit(),
                        Err(SurfaceError::Timeout) => {
                            // Ignore timeouts.
                        }
                    },
                    other => {
                        state.handle_event(other);
                        window.request_redraw();
                        return;
                    }
                };
                state.handle_event(event);
                window.request_redraw();
            }
            _ => {}
        }
    });
}
