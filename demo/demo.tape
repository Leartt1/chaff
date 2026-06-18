# chaff demo tape — render with vhs (https://github.com/charmbracelet/vhs)
#
#   brew install vhs
#   cargo build --release
#   vhs demo/demo.tape        # produces demo/demo.gif
#
Output demo/demo.gif

Set Shell "bash"
Set FontSize 18
Set Width 1400
Set Height 768
Set Padding 20
Set Theme "Dracula"
Set TypingSpeed 55ms

# --- hidden setup: throwaway projects + fake caches, chaff on PATH ---
Hide
Type `export PATH="$PWD/target/release:$PATH"`
Enter
Type "bash demo/setup.sh"
Enter
Sleep 2s
Type "clear"
Enter
Show

# --- scan projects ---
Type "chaff scan /tmp/demo-projects"
Enter
Sleep 3s

# --- and the caches other cleaners miss ---
Type "# ...and the global caches other cleaners miss:"
Enter
Type "HOME=/tmp/demo-home chaff scan --caches /tmp/demo-empty"
Enter
Sleep 4s

# --- interactive clean: pick a couple, reclaim to trash ---
Type "chaff clean /tmp/demo-projects"
Enter
Sleep 1500ms
Down
Sleep 700ms
Space
Sleep 700ms
Down
Sleep 500ms
Space
Sleep 900ms
Enter
Sleep 3s
