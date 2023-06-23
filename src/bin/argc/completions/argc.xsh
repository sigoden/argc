import os
import re
import shutil
from subprocess import Popen, PIPE
from xonsh.completers.tools import RichCompletion
from xonsh.completers.tools import *
from xonsh.completers._aliases import _add_one_completer

def _argc_complete_impl(args):
    if not os.path.exists(args[0]):
        return
    output, _ = Popen(['argc', '--argc-compgen', 'xonsh', *args], stdout=PIPE, stderr=PIPE).communicate()
    candidates = output.decode().split('\n')
    candidates.pop()
    result = set()
    if len(candidates) == 0:
        return result
    if candidates[0] == '__argc_comp:file' or candidates[0] == '__argc_comp:dir':
        return
    for v in candidates:
        parts = v.split('\t')
        value = parts[0]
        if parts[1] == "1":
            value = value + " "
        result.add(RichCompletion(value, display=parts[2], description=parts[3], prefix_len=len(args[-1]), append_closing_quote=False))
    return result

def _argc_complete_locate(cmd):
    if not (cmd in ARGC_SCRIPTS):
        return
    if cmd == 'argc':
        output, _ = Popen(['argc', '--argc-script-path'], stdout=PIPE, stderr=PIPE).communicate()
        return output.decode().split('\n')[0]
    else:
        return shutil.which(cmd)

@contextual_command_completer
def _argc_completer(context):
    if len(context.args) == 0:
        return
    args = [v.value for v in context.args[0:context.arg_index]]
    args.append(context.raw_prefix)
    args.insert(0, _argc_complete_locate(args[0]))
    return _argc_complete_impl(args)

_add_one_completer('argc', _argc_completer, 'start')
