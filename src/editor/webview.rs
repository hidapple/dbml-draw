use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

use crate::error::AppError;
use crate::ir::Diagram;

use super::assets;
use super::ipc;

/// Custom events sent from IPC handler to the event loop.
pub enum UserEvent {
    ExportComplete(String),
}

pub fn run(diagram: Diagram, dbml_path: PathBuf, layout_path: PathBuf) -> Result<(), AppError> {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title(format!(
            "dbml-draw — {}",
            dbml_path.file_name().unwrap_or_default().to_string_lossy()
        ))
        .with_inner_size(tao::dpi::LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)
        .map_err(|e| AppError::EditorError(e.to_string()))?;

    // Build the full HTML with inlined CSS and JS
    let html = assets::EDITOR_HTML
        .replace("/* __EDITOR_CSS__ */", assets::EDITOR_CSS)
        .replace("/* __EDITOR_JS__ */", assets::EDITOR_JS);

    // Serialize diagram data for injection
    let diagram_json = serde_json::to_string(&diagram)
        .map_err(|e| AppError::EditorError(format!("Failed to serialize diagram: {}", e)))?;
    let init_script = format!("window.__INITIAL_DIAGRAM = {};", diagram_json);

    // Wrap diagram in RefCell for interior mutability (IPC handler is Fn, not FnMut)
    let diagram = Rc::new(RefCell::new(diagram));
    let diagram_ipc = Rc::clone(&diagram);
    let dbml_path_ipc = dbml_path.clone();
    let layout_path_ipc = layout_path.clone();

    let webview = WebViewBuilder::new()
        .with_html(&html)
        .with_initialization_script(&init_script)
        .with_ipc_handler(move |message| {
            let body = message.body();
            match ipc::parse_ipc_message(body) {
                Ok(ipc::IpcMessage::TableMoved { table_id, x, y }) => {
                    ipc::handle_table_moved(
                        &mut diagram_ipc.borrow_mut(),
                        &layout_path_ipc,
                        &dbml_path_ipc,
                        &table_id,
                        x,
                        y,
                    );
                }
                Ok(ipc::IpcMessage::SaveLayout { tables }) => {
                    ipc::handle_save_layout(
                        &mut diagram_ipc.borrow_mut(),
                        &layout_path_ipc,
                        &dbml_path_ipc,
                        &tables,
                    );
                }
                Ok(ipc::IpcMessage::ExportPng { data_url }) => {
                    let path = ipc::handle_export_png(&dbml_path_ipc, &data_url);
                    let _ = proxy.send_event(UserEvent::ExportComplete(path));
                }
                Err(e) => {
                    eprintln!("IPC error: {}", e);
                }
            }
        })
        .build(&window)
        .map_err(|e| AppError::EditorError(e.to_string()))?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::ExportComplete(path)) => {
                let js = format!(
                    "window.__onExportComplete({})",
                    serde_json::to_string(&path).unwrap_or_default()
                );
                let _ = webview.evaluate_script(&js);
            }
            _ => {}
        }
    })
}
