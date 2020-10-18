# Meta options
complete -c specsheet -s 'v' -l 'version' -d "Show version of specsheet"
complete -c specsheet -s '?' -l 'help'    -d "Show list of command-line options"

# Running modes
complete -c specsheet -s 'c' -l 'syntax-check'  -d "Don't run, just check the syntax of the input files"
complete -c specsheet -s 'C' -l 'list-commands' -d "Don't run, just list the commands that would be executed"
complete -c specsheet -s 'l' -l 'list-checks'   -d "Don't run, just list the checks that would be run"
complete -c specsheet        -l 'list-tags'     -d "Don't run, just list the tags defined in the documents"
complete -c specsheet        -l 'random-order'  -d "Run the checks in a random order"
complete -c specsheet        -l 'continual'     -d "Run the checks in continual mode, indefinitely"
complete -c specsheet        -l 'delay'         -d "Amount of time to delay between checks" -x
complete -c specsheet        -l 'directory'     -d "Directory to run the tests from" -x -a '(__fish_complete_directories)'
complete -c specsheet -s 'j' -l 'threads'       -d "Number of threads to run in parallel" -x
complete -c specsheet -s 'O' -l 'option'        -d "Set an option or override part of the environment" -x
complete -c specsheet -s 'R' -l 'rewrite'       -d "Add a rule to rewrites values in input documents" -x
complete -c specsheet -s 'z' -l 'analysis'      -d "Run analysis after running checks if there are errors"

# Side process options
complete -c specsheet -s 'x' -l 'exec'          -d "Process to run in the background during execution" -x
complete -c specsheet        -l 'exec-delay' -x -d "Wait an amount of time before running checks"
complete -c specsheet        -l 'exec-port'  -x -d "Wait until a port becomes open before running checks"
complete -c specsheet        -l 'exec-file'  -x -d "Wait until a file exists before running checks"
complete -c specsheet        -l 'exec-line'  -x -d "Wait until the process outputs a line before running checks"
complete -c specsheet        -l 'exec-kill-signal' -x -d "Signal to send to the background process after finishing" -a "
    term\t'Send SIGTERM to stop the process'
    kill\t'Send SIGKILL to stop the process'
"

# Filtering options
complete -c specsheet -s 't' -l 'tags'          -d "Comma-separated list of tags to run" -x
complete -c specsheet        -l 'skip-tags'     -d "Comma-separated list of tags to skip" -x
complete -c specsheet -s 'T' -l 'types'         -d "Comma-separated list of check types to run"  -x -a "apt cmd defaults dns fs group hash homebrew homebrew_cask homebrew_tap http ping specsheet systemd tap tcp udp ufw user"
complete -c specsheet        -l 'skip-types'    -d "Comma-separated list of check types to skip" -x -a "apt cmd defaults dns fs group hash homebrew homebrew_cask homebrew_tap http ping specsheet systemd tap tcp udp ufw user"

# Console output options
complete -c specsheet -s 's' -l 'successes'     -d "How to show successful check results" -x -a "
    hide\t'Do not show successful checks'
    show\t'Show successful checks'
    expand\t'Show successful checks and their steps'
"
complete -c specsheet -s 'f' -l 'failures'      -d "How to show failed check results" -x -a "
    hide\t'Do not show failed checks'
    show\t'Show failed checks'
    expand\t'Show failed checks and their steps'
"
complete -c specsheet        -l 'summaries'     -d "Whether to show the summary lines" -x -a "
    hide\t'Do not show summary lines'
    show\t'Show summary lines'
"
complete -c specsheet -s 'P' -l 'print'         -d "Specify the output format" -x -a "
    ansi\t'Coloured terminal output'
    dots\t'Print one dot per executed check'
    json-lines\t'Print a JSON object per executed check'
    tap\t'Output in Test Anything Protocol format'
"
complete -c specsheet        -l 'color'      -x -d "When to colorise the output" -x -a "
    always\t'Always use colours'
    automatic\t'Use colours when printing to a terminal'
    never\t'Never use colours'
"
complete -c specsheet        -l 'colour'     -x -d "When to colourise the output" -x -a "
    always\t'Always use colours'
    automatic\t'Use colours when printing to a terminal'
    never\t'Never use colours'
"

# Results document options
complete -c specsheet        -l 'html-doc'      -d "Produce an output HTML document" -r
complete -c specsheet        -l 'json-doc'      -d "Produce an output JSON document" -r
complete -c specsheet        -l 'toml-doc'      -d "Produce an output TOML document" -r
