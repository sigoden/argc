import os
import re
import shutil
from subprocess import Popen, PIPE
from xonsh.completers.tools import RichCompletion
from xonsh.completers.tools import *
from xonsh.completers._aliases import _add_one_completer

def _argc_complete_impl(args):
    output, _ = Popen(['argc', '--argc-compgen', 'xonsh', *args], stdout=PIPE, stderr=PIPE).communicate()
    candidates = output.decode().split('\n')
    candidates.pop()
    result = set()
    if len(candidates) == 0:
        result.add(RichCompletion(""))
        return result
    if candidates[0] == '__argc_value:file' or candidates[0] == '__argc_value:dir':
        return result
    for v in candidates:
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
    if not (args[0] in ARGC_SCRIPTS):
        return
    if args[0] == 'argc':
        output, _ = Popen(['argc', '--argc-script-path'], stdout=PIPE, stderr=PIPE).communicate()
        scriptfile = output.decode().split('\n')[0]
    else:
        scriptfile = shutil.which(args[0])
    if not os.path.exists(scriptfile):
        return
    args.insert(0, scriptfile)

    return _argc_complete_impl(args)

_add_one_completer('argc', _argc_completer, 'start')
