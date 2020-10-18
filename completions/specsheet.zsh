#compdef specsheet

__specsheet() {
    _arguments \
        "(- 1 *)"{-v,--version}"[Show version of specsheet]" \
        "(- 1 *)"{-\?,--help}"[Show list of command-line options]" \
        {-c,--syntax-check}"[Don't run, just check the syntax of the input files]" \
        {-C,--list-commands}"[Don't run, just list the commands that would be executed]" \
        {-l,--list-checks}"[Don't run, just list the checks that would be run]" \
        --list-tags"[Don't run, just list the tags defined in the documents]" \
        --random-order"[Run the checks in a random order]" \
        --continual"[Run the checks in continual mode, indefinitely]" \
        --delay"[Amount of time to delay between checks]" \
        --directory"[Directory to run the tests from]" \
        {-j,--threads}"+[Number of threads to run in parallel]" \
        {-O,--option}"[Set an option or override part of the environment]" \
        {-R,--rewrite}"[Add a rule to rewrites values in input documents]" \
        {-z,--analysis}"[Run analysis after running checks if there are errors]" \
        {-x,--exec}"[Process to run in the background during execution]" \
        --exec-delay"[Wait an amount of time before running checks]" \
        --exec-port"[Wait until a port becomes open before running checks]" \
        --exec-file"[Wait until a file exists before running checks]" \
        --exec-line"[Wait until the process outputs a line before running checks]" \
        --exec-kill-signal"[Signal to send to the background process after finishing]:(signal):(term kill)" \
        {-t,--tags}"[Comma-separated list of tags to run]" \
        --skip-tags"[Comma-separated list of tags to skip]" \
        {-T,--types}"[Comma-separated list of check types to run]:(check type):(apt cmd defaults dns fs group hash homebrew homebrew_cask homebrew_tap http ping specsheet systemd tap tcp udp ufw user)" \
        --skip-types"[Comma-separated list of check types to skip]:(check type):(apt cmd defaults dns fs group hash homebrew homebrew_cask homebrew_tap http ping specsheet systemd tap tcp udp ufw user)" \
        {-s,--successes}"[How to show successful check results]:(show option):(hide show expand)" \
        {-f,--failures}"[How to show failed check results]:(show option):(hide show expand)" \
        --summaries"[How to show the summary lines]:(show option):(hide show)" \
        {-P,--print}"[Specify the output format]:(output format):(ansi dots json-lines tap)" \
        {--color,--colour}"[When to use terminal colours]:(output setting):(always automatic never)" \
        --html-doc"[Produce an output HTML document]" \
        --json-doc"[Produce an output JSON document]" \
        --toml-doc"[Produce an output TOML document]" \
        '*:filename:_files'
}

__specsheet
