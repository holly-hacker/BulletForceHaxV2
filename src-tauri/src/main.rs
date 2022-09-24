#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    tauri::Builder::default()
        .register_uri_scheme_protocol(
            "example",
            move |_handle, request| -> Result<tauri::http::Response, Box<dyn std::error::Error>> {
                tauri::http::ResponseBuilder::new()
                    .mimetype("text/plain")
                    .status(200)
                    .body(
                        format!("bf request test: {}", request.uri())
                            .as_bytes()
                            .to_vec(),
                    )
            },
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
