// SPDX-License-Identifier: MIT

fn main() {
    glib_build_tools::compile_resources(
        &["res"],
        "res/resources.gresource.xml",
        "startup-disk.gresource",
    );
}
