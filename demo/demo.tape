# chaff demo tape — render with vhs (https://github.com/charmbracelet/vhs)
#
#   brew install vhs
#   cargo build --release
#   vhs demo/demo.tape        # produces demo/demo.gif
#
Output demo/demo.gif

Set Shell "bash"
Set FontSize 18
Set Width 1080
Set Height 640
Set Padding 18
Set Theme "Dracula"

# --- hidden setup: a realistic junk tree + chaff on PATH ---
Hide
Type `export PATH="$PWD/target/release:$PATH"`
Enter
Type `rm -rf /tmp/chaff_demo && mkdir -p /tmp/chaff_demo/web/node_modules /tmp/chaff_demo/api/.venv /tmp/chaff_demo/cli/target`
Enter
Type `echo '{}' > /tmp/chaff_demo/web/package.json && : > /tmp/chaff_demo/cli/Cargo.toml`
Enter
Type `mkfile -n 420m /tmp/chaff_demo/web/node_modules/dep.bin; mkfile -n 210m /tmp/chaff_demo/cli/target/build.bin; mkfile -n 95m /tmp/chaff_demo/api/.venv/lib.bin`
Enter
Type "clear"
Enter
Show

# --- scan ---
Type "chaff scan /tmp/chaff_demo"
Enter
Sleep 3s

# --- interactive clean: pick a couple, reclaim to trash ---
Type "chaff clean /tmp/chaff_demo"
Enter
Sleep 1500ms
Space
Sleep 700ms
Down
Sleep 500ms
Down
Sleep 500ms
Space
Sleep 900ms
Enter
Sleep 3s
