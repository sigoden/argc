import os
import re
import shutil
from subprocess import Popen, PIPE
from xonsh.completers.tools import RichCompletion
from xonsh.completers.tools import *
from xonsh.completers._aliases import _add_one_completer

@contextual_command_completer
def _argc_completer(context):
    if len(context.args) == 0:
        return
    words = [v.value for v in context.args[0:context.arg_index]]
    words.append(context.prefix)
    cmd = words[0]
    if not (cmd in ARGC_SCRIPTS):
        return
    scriptfile = ""
    if cmd == 'argc':
        output, _ = Popen(['argc', '--argc-script-path'], stdout=PIPE, stderr=PIPE).communicate()
        scriptfile = output.decode().split('\n')[0]
    else:
        scriptfile = shutil.which(cmd)
    if not os.path.exists(scriptfile):
        return
    output, _ = Popen(['argc', '--argc-compgen', 'xonsh', scriptfile, *words], stdout=PIPE, stderr=PIPE).communicate()
    candicates = output.decode().split('\n')
    candicates.pop()
    result = set()
    if len(candicates) == 0:
        return result
    if candicates[0] == '__argc_comp:file' or candicates[0] == '__argc_comp:dir':
        return
    for v in candicates:
        parts = v.split('\t')
        value = parts[0]
        desc = '' if len(parts) == 1 else parts[1]
        result.add(RichCompletion(value, display=value, description=desc, prefix_len=len(context.raw_prefix), append_closing_quote=False))
    return result
    
_add_one_completer('argc', _argc_completer, 'start')
