import os
import re
import shutil
from subprocess import Popen, PIPE
from xonsh.completers.tools import *
from xonsh.completers.path import contextual_complete_path
from xonsh.parsers.completion_context import *
from xonsh.completers._aliases import _add_one_completer

def _argc_complete_path(cur, is_dir=False, fuzz=True):
    if is_dir:
        result = contextual_complete_path(
            CommandContext(
                args=(CommandArg(value='ls'),),
                arg_index=1, prefix=cur,
            ),
            filtfunc=os.path.isdir
        )[0]
    else:
        result = contextual_complete_path(
            CommandContext(
                args=(CommandArg(value='cd'),),
                arg_index=1, prefix=cur,
            ),
        )[0]
    if fuzz == False:
        result = set(filter(lambda x: x.value.startswith(cur), result))

    return result

def _argc_complete_impl(args):
    output, _ = Popen(['argc', '--argc-compgen', 'xonsh', *args], stdout=PIPE, stderr=PIPE).communicate()
    candidates = output.decode().split('\n')
    candidates.pop()
    result = set()
    skip = 0
    if len(candidates) == 0:
        result.add(RichCompletion(""))
        return result

    fuzz = len(candidates) == 1
    if candidates[0] == '__argc_value:file':
        skip = 1
        result = _argc_complete_path(args[-1], fuzz=fuzz,)
    elif  candidates[0] == '__argc_value:dir':
        skip = 1
        result = _argc_complete_path(args[-1], is_dir=True, fuzz=fuzz,)

    for v in candidates[skip:]:
        parts = v.split('\t')
        value = parts[0]
        if parts[1] == "1":
            value = value + " "
        result.add(RichCompletion(value, display=parts[2], description=parts[3], prefix_len=len(args[-1]), append_closing_quote=False))
        
    return result

@contextual_command_completer
def _argc_completer(context):
    if len(context.args) == 0:
        return
    args = [v.value for v in context.args[0:context.arg_index]]
    args.append(context.raw_prefix)

    scriptfile = ""
    if args[0] not in ARGC_SCRIPTS:
        return
    if args[0] == 'argc':
        output, _ = Popen(['argc', '--argc-script-path'], stdout=PIPE, stderr=PIPE).communicate()
        scriptfile = output.decode().split('\n')[0]
    else:
        scriptfile = shutil.which(args[0])
    if scriptfile == "" or scriptfile is None:
        return _argc_complete_path(args[-1])
        
    args.insert(0, scriptfile)

    return _argc_complete_impl(args)

_add_one_completer('argc', _argc_completer, 'start')
