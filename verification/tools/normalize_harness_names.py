#!/usr/bin/env python3
"""Bound Kani's mangled artifact names without changing proof bodies or cases.

Kani 0.68.0 places the complete mangled harness symbol in artifact filenames.
Long fixture/test combinations exceed Linux NAME_MAX during whole-crate list
compilation. Shorten only zero-argument proof names over 80 characters; retain
every full original-case marker and record an auditable, collision-checked alias.
Run this recipe after scenario generators and before discovery/verification.
"""
import hashlib
import json
from pathlib import Path
import re

ROOT=Path(__file__).resolve().parents[2]
OUTPUT=ROOT/'verification/cases/harness_names.json'

def shortened(module,name):
    if len(name)<=80:return name
    digest=hashlib.sha256((module+'::'+name).encode()).hexdigest()[:24]
    return name[:48]+'__'+digest

def main():
    records={}
    if OUTPUT.exists():
        old=json.loads(OUTPUT.read_text())
        records={(r['file'],r['original_entry']):r for r in old['aliases']}
    count=0
    for path in sorted((ROOT/'verification/kani/src').glob('*.rs')):
        relative=path.relative_to(ROOT).as_posix();text=path.read_text()
        functions=list(re.finditer(r'^fn (\w+)\(\)\s*\{',text,re.M))
        names={m[1] for m in functions}
        for match in reversed(functions):
            name=match[1];new=shortened(path.stem,name)
            if name==new:continue
            if new in names:raise ValueError('Harness alias collision: '+path.stem+'::'+new)
            # No helper/call target is eligible. This only changes a named
            # proof entry, never an internal operation or assertion.
            prefix=text[max(0,match.start()-300):match.start()]
            if not re.search(r'#\[(?:cfg_attr\(kani,\s*kani::proof\)|kani::proof)\]',prefix):
                raise ValueError('Long non-proof function requires separate review: '+name)
            if len(re.findall(r'\b'+re.escape(name)+r'\b',text))!=1:
                raise ValueError('Proof function has other references: '+name)
            markers=re.findall(r'^// original-case: (\S+)\s*$',text[:match.start()],re.M)
            if not markers:raise ValueError('Long proof lacks exact original identity')
            text=text[:match.start(1)]+new+text[match.end(1):]
            records[(relative,path.stem+'::'+name)]={
                'file':relative,'original_case':markers[-1],
                'original_entry':path.stem+'::'+name,'entry':path.stem+'::'+new,
            }
            names.add(new);count+=1
        path.write_text(text)
    for r in records.values():
        p=ROOT/r['file'];text=p.read_text()
        if not re.search(r'^fn '+re.escape(r['entry'].split('::')[-1])+r'\(\)',text,re.M):
            raise ValueError('Saved canonical proof name no longer present: '+r['entry'])
        r['proof_source_sha256']=hashlib.sha256(p.read_bytes()).hexdigest()
    aliases=sorted(records.values(),key=lambda r:(r['file'],r['original_entry']))
    if len({r['entry'] for r in aliases})!=len(aliases):raise ValueError('Duplicate canonical entry')
    OUTPUT.write_text(json.dumps({'schema':1,'kani_version':'0.68.0',
        'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'compiler_authority':'https://github.com/model-checking/kani/blob/kani-0.68.0/kani-compiler/src/codegen_cprover_gotoc/compiler_interface.rs',
        'notice':'Naming integration only; bodies, domains, markers, assertions and coverage are unchanged. No proof credit.',
        'aliases':aliases},indent=2)+'\n')
    print(json.dumps({'shortened_now':count,'preserved_aliases':len(aliases),'path':str(OUTPUT)}))

if __name__=='__main__':main()
