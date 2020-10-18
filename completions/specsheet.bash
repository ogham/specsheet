_specsheet()
{
    cur=${COMP_WORDS[COMP_CWORD]}
    prev=${COMP_WORDS[COMP_CWORD-1]}

    case "$prev" in
        -'?'|--help|-v|--version)
            return
            ;;

        -j|--threads)
            COMPREPLY=( $( compgen -W '{0..9}' -- "$cur" ) )
            return
            ;;

        -T|--types|--skip-types)
            COMPREPLY=( $( compgen -W 'apt cmd defaults dns fs group hash homebrew homebrew_cask homebrew_tap http ping specsheet systemd tap tcp udp ufw user' -- "$cur" ) )
            return
            ;;

        -s|--successes|-f|--failures)
            COMPREPLY=( $( compgen -W 'hide show expand' -- $cur ) )
            return
            ;;

        --summaries)
            COMPREPLY=( $( compgen -W 'hide show' -- $cur ) )
            return
            ;;

        --exec-kill-signal)
            COMPREPLY=( $( compgen -W 'term kill' -- $cur ) )
            return
            ;;

        -P|--print)
            COMPREPLY=( $( compgen -W 'ansi docs json-lines tap' -- $cur ) )
            return
            ;;

        --color|--colour)
            COMPREPLY=( $( compgen -W 'always automatic never' -- $cur ) )
            return
            ;;
    esac

    case "$cur" in
        -*)
            COMPREPLY=( $( compgen -W '$( _parse_help "$1" )' -- "$cur" ) )
            ;;

        *)
            _filedir
            ;;
    esac
} &&
complete -o filenames -o bashdefault -F _specsheet specsheet
