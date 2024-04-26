from subprocess import Popen, PIPE
from xonsh.completers.tools import *
from xonsh.completers._aliases import _add_one_completer

@contextual_command_completer
def _argc_completer(context):
    if len(context.args) == 0:
        return
    args = [v.value for v in context.args[0:context.arg_index]]
    args.append(context.raw_prefix)

    if args[0] not in __xonsh__.env['ARGC_XONSH_SCRIPTS']:
        return

    output, _ = Popen(['argc', '--argc-compgen', 'xonsh', '', *args], stdout=PIPE, stderr=PIPE).communicate()
    candidates = output.decode().split('\n')
    candidates.pop()
    result = set()
    skip = 0
    if len(candidates) == 0:
        result.add(RichCompletion(""))
        return result

    for v in candidates:
        parts = v.split('\t')
        value = parts[0]
        if parts[1] == "1":
            value = value + " "
        result.add(RichCompletion(value, display=parts[2], description=parts[3], prefix_len=len(args[-1]), append_closing_quote=False))
        
    return result

if 'ARGC_XONSH_SCRIPTS' not in __xonsh__.env:
    __xonsh__.env['ARGC_XONSH_SCRIPTS'] = []
    
if 'argc' not in __xonsh__.completers:
    _add_one_completer('argc', _argc_completer, 'start')
