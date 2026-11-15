"""The stack through a routine, worked out statically (tools/recomp).

For every instruction of a routine: how far sp is below where it was at
the routine's entry (0 there; the return address is at 0 from the
caller's side).  A push lowers it, a pop raises it, `movem` by its
registers, `link`/`unlk`, `addq`/`subq`/`lea`/`adda` on sp by their
amount; a call leaves it as it was (the callee returns what it took, the
caller drops the arguments it pushed).  The depth must agree wherever
paths meet, and be 0 at every return; a routine where it can't be told
(sp from a register, `move.l` into sp) is reported.
"""
from recomp.routines import successors


# where the game returns with something still on the stack (rts then
# jumps to it): the port returns instead (docs/PORT.md, "Bugs")
BUGS = {
    # func_00D652: when func_001222 can't make the task (`bcs.w`), d0 is
    # still pushed
    0xD6DA: 'returns with d0 still pushed when func_001222 fails',
}


class StackError(Exception):
    def __init__(self, msg, tail=None):
        super().__init__(msg)
        # a tail call made with something on the stack: its target
        self.tail = tail


def size_of(mn):
    if mn.endswith('.b'):
        return 1
    if mn.endswith('.w'):
        return 2
    if mn.endswith('.l'):
        return 4
    return None


def effect(i, depth):
    """sp's depth after instruction i, given it before (or StackError)."""
    mn = i.mn
    sz = size_of(mn)
    delta = 0
    for k, o in enumerate(i.ops):
        if o.reg != 7:
            continue
        if o.kind == 'predec':
            if mn.startswith('movem'):
                n = bin(i.ops[0].val).count('1')
                delta += n * sz
            else:
                delta += max(sz or 2, 2)
        elif o.kind == 'postinc':
            if mn.startswith('movem'):
                n = bin(i.ops[1].val).count('1')
                delta -= n * sz
            else:
                delta -= max(sz or 2, 2)
    dst = i.ops[-1] if i.ops else None
    if dst is not None and dst.kind == 'an' and dst.reg == 7:
        src = i.ops[0]
        if mn in ('addq.w', 'addq.l', 'adda.w', 'adda.l') and \
                src.kind in ('quick', 'imm'):
            delta -= src.val
        elif mn in ('subq.w', 'subq.l', 'suba.w', 'suba.l') and \
                src.kind in ('quick', 'imm'):
            delta += src.val
        elif mn == 'lea' and src.kind == 'disp' and src.reg == 7:
            delta -= src.val
        elif mn == 'movem.l':
            pass
        else:
            raise StackError('sp set by %s' % i.format())
    if mn == 'link':
        delta += 4 - i.ops[1].val
    return depth + delta


def aliases_after(i, dep, al):
    """Address registers that point into the stack after i: {an: the
    position (bytes below the routine's entry) they point at}.  `link an`
    and `movea.l sp,an` make one, `adda`/`addq`/`subq`/`lea (d,an)` move
    it, anything else written to it ends it."""
    al = dict(al)
    mn = i.mn
    ops = i.ops
    if mn == 'link':
        al[ops[0].reg] = dep + 4
        return al
    if mn == 'movea.l' and ops[0].kind == 'an' and ops[0].reg == 7:
        al[ops[1].reg] = dep
        return al
    dst = ops[-1] if ops else None
    if dst is not None and dst.kind == 'an' and dst.reg in al:
        n = dst.reg
        src = ops[0]
        if mn in ('adda.w', 'adda.l', 'addq.w', 'addq.l') and \
                src.kind in ('imm', 'quick'):
            al[n] -= src.val
        elif mn in ('suba.w', 'suba.l', 'subq.w', 'subq.l') and \
                src.kind in ('imm', 'quick'):
            al[n] += src.val
        elif mn == 'lea' and src.kind == 'disp' and src.reg == n:
            al[n] -= src.val
        else:
            del al[n]
    if mn == 'unlk':
        al.pop(ops[0].reg, None)
    if mn.startswith('movem') and ops[1].kind == 'reglist':
        for k in range(8, 15):
            if ops[1].val >> k & 1:
                al.pop(k - 8, None)
    return al


def depths(d, body):
    """{address: depth before it} for a routine's instructions (body, the
    first its entry), and {address: aliases before it} (aliases_after):
    `unlk an` brings sp back to where `link` found it."""
    entry = body[0]
    inside = set(body)
    out = {entry: 0}
    alias = {entry: {}}
    todo = [entry]
    while todo:
        a = todo.pop()
        i = d.insns[a]
        dep = out[a]
        al = alias[a]
        if i.mn == 'unlk':
            if i.ops[0].reg not in al:
                raise StackError('unlk without link at %06X' % a)
            after = al[i.ops[0].reg] - 4
        else:
            after = effect(i, dep)
        al_after = aliases_after(i, dep, al)
        # a return deeper than the entry pops the caller's return address
        # (and what the caller saved): it returns from the caller too.
        # Above it, only the game's bugs (BUGS)
        if i.flow == 'ret' and i.mn == 'rts' and 0 < after and \
                a not in BUGS:
            raise StackError('rts at depth %d at %06X' % (after, a))
        for s, _ in successors(d, a):
            if s not in inside:
                if after > 0:
                    raise StackError('tail call at depth %d at %06X' %
                                     (after, a), tail=s)
                continue
            if s in out:
                if out[s] != after:
                    # only a return may be reached at two depths: with
                    # the return address popped (the routine returns from
                    # its caller too: stack.py's `unwind`) and without
                    j = d.insns[s]
                    if not (j.mn == 'rts' and (s in BUGS or all(
                            x <= -4 or x == 0 for x in (out[s], after)))):
                        raise StackError('depths %d and %d meet at %06X' %
                                         (out[s], after, s))
                if alias[s] != al_after:
                    alias[s] = {k: v for k, v in alias[s].items()
                                if al_after.get(k) == v}
                continue
            out[s] = after
            alias[s] = al_after
            todo.append(s)
    return out, alias
