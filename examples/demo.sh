# @describe A demo cli

# @cmd Upload a file
# @arg target!                      File to upload
upload() {
    echo "cmd                       upload"
    echo "arg:  target              $argc_target"
}

# @cmd Download a file
# @flag     -f --force              Override existing file
# @option   -t --tries <NUMBER>     Set number of retries to NUMBER
# @arg      source!                 Url to download from
# @arg      target                  Save file to
download() {
    echo "cmd:                      download"
    echo "flag:   --force           $argc_force"
    echo "option: --tries           $argc_tries"
    echo "arg:    source            $argc_source"
    echo "arg:    target            $argc_target"
}

eval "$(argc -e $0 "$@")"
