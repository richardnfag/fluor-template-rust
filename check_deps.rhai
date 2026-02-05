let tools = [
    #{"cmd": "cargo-component", "install": "cargo install cargo-component"},
    #{"cmd": "just", "install": "cargo install just"}
];

for tool in tools {
    let check_failed = false;

    try {
        system::command(tool.cmd, ["--version"]);
    } catch (err) {
        check_failed = true;
    }

    if check_failed {
        print("⚠️  '" + tool.cmd + "' was not found.");

        if variable::confirm("Would you like to install '" + tool.cmd + "' now?") {
            print("⚡ Installing " + tool.cmd + " via cargo install...");

            try {
                system::command("cargo", ["install", tool.cmd.split(" ")[0]]);
                print("⚡ " + tool.cmd + " installed successfully!");
            } catch (err) {
                print("❌ Failed to install " + tool.cmd + ": " + err);
                throw "Installation failed";
            }
        } else {
            throw "Aborted: " + tool.cmd + " is required for this template.";
        }
    }
}
