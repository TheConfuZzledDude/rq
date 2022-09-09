// #[cfg(target_os = "linux")]
// pub fn get_mouse_position(window: Window) {
//     let (tx, rx) = mpsc::channel::<Point>();
//     let thread_tx = tx.clone();
//     let thread_window = window.clone();
//     window.run_on_main_thread(|| {
//         let window = thread_window.gtk_window().window();
//     });
// }

// #[cfg(target_os = "macos")]
// #[repr(C)]
// #[derive(Copy, Clone, Debug)]
// pub struct NSPoint {
//     x: f64,
//     y: f64,
// }

// #[cfg(target_os = "macos")]
// pub fn get_mouse_position(window: Window) -> tauri::LogicalPosition<f64> {
//     use objc::{msg_send, sel, sel_impl};
//     use std::{
//         sync::{Arc, Mutex, mpsc},
//     };

//     use tauri::LogicalPosition;

//     let (tx, rx) = mpsc::channel::<NSPoint>();

//     let thread_tx = tx.clone();
//     window
//         .with_webview(move |webview| unsafe {
//             // *b.clone().lock().expect("Couldn't lock mutex") =
//             let mut point: NSPoint = msg_send![webview.ns_window(), mouseLocationOutsideOfEventStream];
//             // point = msg_send![webview.ns_window(), convertPointToBacking:point];
//             thread_tx.send(point).expect("Couldnt send on channel");
//         })
//         .unwrap();

//     let NSPoint { x, y } = rx.recv().unwrap();
//     println!("Phyiscal Size: {:?}, Logical Size: {:?}", window.inner_size().unwrap(), window.inner_size().unwrap().to_logical::<f64>(window.scale_factor().unwrap()));
//     LogicalPosition::new(x, window.inner_size().unwrap().to_logical::<f64>(window.scale_factor().unwrap()).height - y)
// }
