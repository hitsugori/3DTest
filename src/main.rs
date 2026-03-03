// SPDX-License-Identifier: GPL-3.0-or-later
// 3DTest — wgpu multi-backend 3D renderer
// Copyright (C) 2026 mikedev_ <mike@mikeden.site>
//
// GPL-3.0-or-later — see COPYING or <https://www.gnu.org/licenses/>
// pssss! those anti-ai people: debugging was done by Claude Opus 4.6 but all of the code was written by me >w<
#![windows_subsystem = "windows"]

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn")
    ).init();

    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        eprintln!("PANIC: {msg}");
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("3DTest crashed")
            .set_description(&format!(
                "An unexpected error occurred:\n\n{msg}\n\nRun with RUST_LOG=debug for more detail."
            ))
            .show();
    }));

    pollster::block_on(threedtest::run());
}
