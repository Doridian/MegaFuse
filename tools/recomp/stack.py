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


class StackError(Exception):
    pass


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
    if mn == 'unlk':
        raise StackError('unlk')        # handled by the caller (frames)
    return depth + delta


def depths(d, body):
    """{address: depth before it} for a routine's instructions (body, the
    first its entry), following the frame of `link an`: `unlk an` brings
    sp back to where `link` found it."""
    entry = body[0]
    inside = set(body)
    out = {entry: 0}
    frames = {}                    # an -> depth before its link
    todo = [entry]
    while todo:
        a = todo.pop()
        i = d.insns[a]
        dep = out[a]
        if i.mn == 'link':
            frames[i.ops[0].reg] = dep
        if i.mn == 'unlk':
            if i.ops[0].reg not in frames:
                raise StackError('unlk without link at %06X' % a)
            after = frames[i.ops[0].reg]
        else:
            after = effect(i, dep)
        if i.flow == 'ret' and after != 0 and i.mn == 'rts':
            raise StackError('rts at depth %d at %06X' % (after, a))
        for s, _ in successors(d, a):
            if s not in inside:
                if after != 0:
                    raise StackError('tail call at depth %d at %06X' %
                                     (after, a))
                continue
            if s in out:
                if out[s] != after:
                    raise StackError('depths %d and %d meet at %06X' %
                                     (out[s], after, s))
                continue
            out[s] = after
            todo.append(s)
    return out
