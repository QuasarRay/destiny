#!/usr/bin/env python3
"""Move complete evolution cases into a bounded source module, byte for byte.

The shared prelude is the reviewed park generator prelude. Qualified identities
and bodies are preserved; fully qualified Rust module paths change and require
fresh mappings and proof evidence. No assurance fields are changed here.
"""
from pathlib import Path
import importlib.util
import re

ROOT=Path(__file__).resolve().parents[2]

def main():
    spec=importlib.util.spec_from_file_location('park_generator',ROOT/'verification/tools/generate_park_harnesses.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    old=ROOT/'verification/kani/src/park_contracts.rs'
    new=ROOT/'verification/kani/src/park_evolution_contracts.rs'
    text=old.read_text();markers=list(re.finditer(r'^// original-case: (\S+)\n',text,re.M))
    kept=[];moved=[]
    for i,marker in enumerate(markers):
        end=markers[i+1].start() if i+1<len(markers) else len(text)
        block=text[marker.start():end]
        (moved if '/ballpark/evolve/' in marker[1] else kept).append(block)
    if not moved:raise ValueError('No evolution bodies to move; do not repeat a completed split')
    if new.exists():raise ValueError('Destination exists; preserve it and inspect before changing layout')
    existing=text[:markers[0].start()]+''.join(kept)
    destination=module.PREAMBLE+'\n'+''.join(moved)
    before=[m[1] for m in markers]
    after=re.findall(r'^// original-case: (\S+)$',existing+'\n'+destination,re.M)
    if sorted(before)!=sorted(after) or len(after)!=len(set(after)):raise ValueError('Qualified case inventory changed')
    # Every moved/retained body is the same original byte sequence.
    if any(block not in destination for block in moved) or any(block not in existing for block in kept):raise ValueError('Case body changed')
    old.write_text(existing);new.write_text(destination)
    print('Preserved',len(before),'case bodies; moved',len(moved),'evolution cases; sizes',len(existing.encode()),len(destination.encode()))

if __name__=='__main__':main()
