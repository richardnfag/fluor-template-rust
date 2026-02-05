let name = "{{project-name}}";

print("Generating bindings for " + name + "...");

try {
    system::command("cargo", ["component", "bindings"]);
    system::command("mkdir", ["-p", "out"]);

    print("⚡ Project " + name + " is ready and bindings are generated!");
} catch (err) {
    print("⚠️  Warning: Could not generate bindings automatically.");
    print("Try running 'cargo component bindings' manually.");
    print(err);
}
