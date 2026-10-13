#!/usr/bin/env python3
"""Generate reviewed server action constructors and one original scenario.

This translates a bounded AST grammar from immutable Git objects. Unsupported
native construction/stream/callback syntax is an explicit error, not an empty
harness. Invoke from the repository root with an active fixture-qualified packet.
"""
import argparse
import ast
import hashlib
from pathlib import Path
import subprocess

ROOT = Path.cwd()
REV = "ef738406b61ad06bae2f402857e464735ab106a2"
TEST_SOURCE = "python/destiny/test/net/server/test_actions.py"
MODEL_SOURCE = "python/destiny/net/server/_actions.py"
PREFIX = TEST_SOURCE + "::TestActions::"
MODEL_TARGET = ROOT / "verification/spec/src/server_actions.rs"
PROOF_TARGET = ROOT / "verification/kani/src/server_actions_contracts.rs"
BASE = ROOT / "verification/tools/server_actions_base.rs"


def original(path):
    return subprocess.run(["git", "show", f"{REV}:{path}"], cwd=ROOT, check=True,
                          capture_output=True, text=True).stdout


def methods(text):
    cls = next(x for x in ast.parse(text).body if isinstance(x, ast.ClassDef) and x.name == "Actions")
    return {x.name: x for x in cls.body if isinstance(x, ast.FunctionDef)}


def simple_constructor(node):
    body = [x for x in node.body if not (isinstance(x, ast.Expr) and isinstance(x.value, ast.Constant) and isinstance(x.value.value, str))]
    orientation_guard = False
    if len(body) == 2 and isinstance(body[0], ast.If):
        guard = body.pop(0)
        if ast.unparse(guard.test) != "not is_dynamical_orientation_enabled()" or guard.orelse or len(guard.body) != 1 or not isinstance(guard.body[0], ast.Raise):
            return None
        raised = guard.body[0].exc
        if not isinstance(raised, ast.Call) or not isinstance(raised.func, ast.Name) or raised.func.id != "RuntimeError":
            return None
        orientation_guard = True
    if len(body) != 1 or not isinstance(body[0], ast.Expr) or not isinstance(body[0].value, ast.Call):
        return None
    call = body[0].value
    if ast.unparse(call.func) != "self._add_to_system_history":
        return None
    if len(call.args) < 2 or not isinstance(call.args[0], ast.Name) or not isinstance(call.args[1], ast.Constant) or not isinstance(call.args[1].value, str):
        raise ValueError("Unexpected direct action constructor")
    arguments = [x.arg for x in node.args.args[1:]]
    if any(not isinstance(x, ast.Name) or x.id not in arguments for x in call.args[2:]):
        raise ValueError("Unreviewed constructor arguments")
    if call.args[0].id != arguments[0] or call.args[0].id != "src_id":
        raise ValueError("Unreviewed source ball placement")
    local = "None"
    insert = "true"
    for keyword in call.keywords:
        if keyword.arg == "local_args" and isinstance(keyword.value, ast.Name) and keyword.value.id in arguments:
            local = f"Some({keyword.value.id})"
        elif keyword.arg == "insert_ball_id_into_args" and isinstance(keyword.value, ast.Constant) and isinstance(keyword.value.value, bool):
            insert = str(keyword.value.value).lower()
        else:
            raise ValueError("Unreviewed constructor keyword")
    guard_rust = "        if !self.dynamical_orientation { return Err(NetworkActionError::DynamicalOrientationDisabled); }\n" if orientation_guard else ""
    rust = f"    // original-definition: {MODEL_SOURCE}:{node.lineno}\n    pub fn {node.name}(&mut self, " + ", ".join(f"{name}: ActionValue" for name in arguments) + ") -> Result<(), NetworkActionError> {\n" + guard_rust
    rust += f"        self.add_to_system_history(src_id, ActionName::{call.args[1].value}, &[" + ", ".join(x.id for x in call.args[2:]) + f"], {insert}, {local})\n    }}\n"
    return rust


def prepare_model(target, base):
    text = original(MODEL_SOURCE)
    definitions = methods(text)
    constructors = [simple_constructor(n) for n in definitions.values()]
    constructors = [x for x in constructors if x is not None]
    names = sorted({x.args[1].value for x in ast.walk(ast.parse(text)) if isinstance(x, ast.Call)
                    and ast.unparse(x.func) == "self._add_to_system_history" and len(x.args) >= 2
                    and isinstance(x.args[1], ast.Constant) and isinstance(x.args[1].value, str)})
    # These protocol names are emitted by the original batcher rather than
    # Actions constructors. Retain their actual source dependency and names.
    batcher=ast.parse(original("python/destiny/net/server/_parkupdatebatcher.py"))
    emitted={node.elts[0].value for node in ast.walk(batcher) if isinstance(node,ast.Tuple)
        and node.elts and isinstance(node.elts[0],ast.Constant) and isinstance(node.elts[0].value,str)}
    extra={"SetState","RemoveBalls","PackagedAction"}
    if not extra<=emitted:raise ValueError("Reviewed batcher protocol names changed")
    ticker=ast.parse(original("python/destiny/net/server/_ticker.py"))
    ticker_names={node.elts[0].value for node in ast.walk(ticker) if isinstance(node,ast.Tuple)
        and node.elts and isinstance(node.elts[0],ast.Constant) and isinstance(node.elts[0].value,str)}
    if "RemoveBall" not in ticker_names:raise ValueError("Reviewed ticker removal protocol changed")
    extra.add("RemoveBall")
    names=sorted(set(names)|extra)
    if any(not name.isidentifier() for name in names):
        raise ValueError("Protocol names need reviewed Rust identifiers")
    contents = base.read_text()
    contents += "\n// original-source-sha256: " + hashlib.sha256(text.encode()).hexdigest() + "\n"
    contents += "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum ActionName {\n" + "\n".join("    " + n + "," for n in names) + "\n}\n\n"
    contents += "impl NetworkActions {\n" + "\n".join(constructors) + "\n"
    contents += """    // original-definition: python/destiny/net/server/_actions.py:475
    pub fn set_ball_harmonic(&mut self, src_id: ActionValue, harmonic_value: ActionValue,
        corporation_id: ActionValue, alliance_id: ActionValue, is_forcefield: ActionValue)
        -> Result<(), NetworkActionError>
    {
        if is_forcefield.truthy() {
            let massive = harmonic_value != ActionValue::Integer(-1)
                || corporation_id != ActionValue::Integer(-1)
                || alliance_id != ActionValue::Integer(-1);
            self.set_ball_massive(src_id, ActionValue::Boolean(massive))?;
        }
        self.add_to_system_history(src_id, ActionName::SetBallHarmonic,
            &[harmonic_value, corporation_id, alliance_id, is_forcefield], true, None)
    }
}
"""
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(contents)
    print("Translated", len(constructors), "reviewed direct constructors and forcefield ordering")


PREAMBLE = """// Copyright © 2023 CCP ehf. MIT license, retained from original Destiny.
// Generated from the pinned original by generate_server_action_harnesses.py.
// Model bounds: eight actions, eight parameters/action, four ball projections;
// these concrete empty-park scenarios need at most three actions. Primitive
// values retain their original integer/float/bool types. No kani::assume.

use destiny_original_spec::{ActionName, ActionValue as V, BoundedList, NetworkAction,
    NetworkActionError, NetworkActionHistory, NetworkActionRecord, NetworkActions,
    OriginalParkProjection, OriginalBallInit, NetworkBallProjection, Vec3};

// Input adapter for the reviewed native-fixture state projection. Every
// created ID is listed by the caller. Native constructor/Blue/spatial and
// Python representation correspondence remain required, never assumed.
fn refresh_fixture_balls(actions: &mut NetworkActions, park: &OriginalParkProjection, ids: &[i64]) {
    actions.balls=BoundedList::new();
    let mut i=0_usize;
    while i<ids.len() {
        if let Some(id)=ids.get(i) {
            if park.contains(*id) {
                let ball=park.retained(*id);
                assert!(actions.balls.push(NetworkBallProjection {
                    id:V::Integer(ball.id),old_bubble_id:ball.old_bubble,
                    new_bubble_id:ball.new_bubble,cloak_mode:i64::from(ball.is_cloaked),
                    is_interactive:ball.is_interactive,speed_fraction:f64::from(ball.speed_fraction),
                }));
            }
        }
        i=i.saturating_add(1);
    }
}

const BALL_ID_1: V = V::Integer(1_000_000_000_001);
const BALL_ID_2: V = V::Integer(1_000_000_000_002);
const BALL_ID_3: V = V::Integer(1_000_000_000_003);
const CORP_ID_1: V = V::Integer(98_000_001);
const ALLIANCE_ID_1: V = V::Integer(99_000_001);
const INITIAL_TIMESTAMP: i64 = 0;

fn action(name: ActionName, arguments: &[V], local_argument: Option<V>) -> NetworkAction {
    let mut parameters = BoundedList::new();
    let mut i: usize = 0;
    while i < arguments.len() {
        if let Some(value) = arguments.get(i) { assert!(parameters.push(*value)); }
        i = i.saturating_add(1);
    }
    NetworkAction { name, parameters, local_argument, stream:None }
}

fn records(entries: &[(V, i64, NetworkAction)]) -> NetworkActionHistory {
    let mut result = NetworkActionHistory::new();
    let mut i: usize = 0;
    while i < entries.len() {
        if let Some((ball_id, timestamp, action)) = entries.get(i) {
            assert!(result.push(NetworkActionRecord { ball_id: *ball_id,
                timestamp: *timestamp, action: *action }));
        }
        i = i.saturating_add(1);
    }
    result
}

fn contains_record(history: NetworkActionHistory, ball_id: V, timestamp: i64,
    action: NetworkAction) -> bool
{
    let mut i: usize = 0;
    while i < history.len() {
        if history.get(i) == Some(NetworkActionRecord { ball_id, timestamp, action }) { return true; }
        i = i.saturating_add(1);
    }
    false
}

"""


class Translator:
    def __init__(self, definitions):
        self.definitions = definitions
        self.kinds = {name: "value" for name in ("BALL_ID_1", "BALL_ID_2", "BALL_ID_3", "CORP_ID_1", "ALLIANCE_ID_1")}
        self.kinds["INITIAL_TIMESTAMP"] = "timestamp"
        self.native_balls = []
        self.helper = next(n for n in ast.parse(original("python/destiny/test/net/server/helpers.py")).body
                           if isinstance(n,ast.FunctionDef) and n.name=="add_ball_to_park")

    def scalar(self,node,kind):
        if isinstance(node,ast.Constant) and type(node.value) in (int,float,bool):
            if kind=="bool": return str(bool(node.value)).lower()
            if kind=="i64" and type(node.value) is not int: raise ValueError("Unreviewed integer fixture input")
            return str(float(node.value))+"_"+kind if kind in ("f32","f64") else str(node.value)+"_i64"
        if isinstance(node,ast.UnaryOp) and isinstance(node.op,ast.USub):
            return "-("+self.scalar(node.operand,kind)+")"
        if isinstance(node,ast.Attribute) and isinstance(node.value,ast.Name) and node.value.id in self.native_balls and node.attr=="id" and kind=="i64":
            return node.value.id
        raise ValueError("Unreviewed native fixture scalar: "+ast.unparse(node))

    def native_creation(self,name,call):
        if len(call.args)!=1 or ast.unparse(call.args[0])!="self._park":
            raise ValueError("Unreviewed native helper target")
        names=[n.arg for n in self.helper.args.args[1:]]
        defaults=dict(zip(names,self.helper.args.defaults))
        supplied={k.arg:k.value for k in call.keywords}
        if len(supplied)!=len(call.keywords) or set(supplied)-set(names): raise ValueError("Unreviewed helper keywords")
        values={**defaults,**supplied}
        fields={"id":("ball_id","i64"),"mass":("mass","f64"),"radius":("radius","f32"),
            "max_velocity":("max_velocity","f32"),"is_free":("is_free","bool"),
            "is_global":("is_global","bool"),"is_massive":("is_massive","bool"),
            "is_interactive":("is_interactive","bool"),"is_space_junk":("is_space_junk","bool"),
            "agility":("agility","f32"),"speed_fraction":("speed_fraction","f32")}
        if set(supplied)&{"maxAngularSpeed","angularAgility"}: raise ValueError("Dynamical fixture branch requires its own translation")
        members=[f"{field}: {self.scalar(values[argument],kind)}" for field,(argument,kind) in fields.items()]
        for label,axes in (("position",("x","y","z")),("velocity",("vx","vy","vz"))):
            members.append(label+": Vec3 { "+", ".join(f"{axis}: {self.scalar(values[argument],'f64')}" for axis,argument in zip("xyz",axes))+" }")
        self.native_balls.append(name)
        self.kinds[name]="ball-id"
        return ["assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall",
                f"let {name}=park.add_ball(OriginalBallInit {{ "+", ".join(members)+" });",
                f"assert!(park.contains({name}) && !park.scope_error); // original helper must create the object"]

    def expression(self, node):
        if isinstance(node, ast.Name) and node.id in self.kinds:
            return node.id, self.kinds[node.id]
        if isinstance(node, ast.Constant):
            value = node.value
            if type(value) is bool: return f"V::Boolean({str(value).lower()})", "value"
            if type(value) is int: return f"V::Integer({value})", "value"
            if type(value) is float: return f"V::Float({value!r})", "value"
        if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub) and isinstance(node.operand, ast.Constant):
            return self.expression(ast.Constant(-node.operand.value))
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add) and isinstance(node.left, ast.Name) and node.left.id == "INITIAL_TIMESTAMP" and isinstance(node.right, ast.Constant) and type(node.right.value) is int:
            return f"INITIAL_TIMESTAMP.saturating_add({node.right.value})", "timestamp"
        if (isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add)
                and isinstance(node.left, ast.Attribute) and node.left.attr == "id"
                and isinstance(node.left.value, ast.Name) and node.left.value.id in self.native_balls
                and isinstance(node.right, ast.Constant) and type(node.right.value) is int):
            # Concrete allocated IDs and the literal offset fit i64. Rust's
            # overflow check remains enabled; arbitrary Python integers are
            # outside this representation correspondence.
            return f"V::Integer({node.left.value.id} + {node.right.value}_i64)", "value"
        if isinstance(node, ast.Attribute) and ast.unparse(node) == "destiny.DSTNORMALCLOAK":
            return "V::Integer(1)", "value"
        if isinstance(node,ast.Attribute) and isinstance(node.value,ast.Name) and node.value.id in self.native_balls:
            ball=node.value.id
            if node.attr=="id": return f"V::Integer({ball})", "value"
            if node.attr in {"oldBubbleId","newBubbleId"}:
                return f"V::Integer(park.retained({ball})."+{"oldBubbleId":"old_bubble","newBubbleId":"new_bubble"}[node.attr]+")", "value"
            raise ValueError("Unreviewed projected native member")
        if isinstance(node, ast.Tuple) and len(node.elts) in (2, 3) and isinstance(node.elts[0], ast.Constant) and isinstance(node.elts[0].value, str) and isinstance(node.elts[1], ast.Tuple):
            arguments = [self.expression(x) for x in node.elts[1].elts]
            if any(kind != "value" for _, kind in arguments): raise ValueError("Unexpected protocol argument")
            local = "None" if len(node.elts) == 2 else "Some(" + self.expression(node.elts[2])[0] + ")"
            return f"action(ActionName::{node.elts[0].value}, &[" + ", ".join(code for code, _ in arguments) + f"], {local})", "action"
        if isinstance(node, ast.List):
            entries = []
            for entry in node.elts:
                if not isinstance(entry, ast.Tuple) or len(entry.elts) != 3: raise ValueError("Unreviewed action record")
                ball, time, event = [self.expression(x) for x in entry.elts]
                if [ball[1], time[1], event[1]] != ["value", "timestamp", "action"]: raise ValueError("Unexpected history tuple types")
                entries.append(f"({ball[0]}, {time[0]}, {event[0]})")
            return "records(&[" + ", ".join(entries) + "])", "history"
        if isinstance(node, ast.Call):
            return self.operation(node)
        raise ValueError("Unreviewed expression: " + ast.dump(node))

    def operation(self, call):
        name = ast.unparse(call.func)
        if name=="self._park.SetBallFree" and len(call.args)==2 and not call.keywords:
            return f"park.set_free({self.scalar(call.args[0],'i64')}, {self.scalar(call.args[1],'bool')})", "unit"
        if not name.startswith("self._actions."): raise ValueError("Native or unreviewed call: " + name)
        method = name.rsplit(".", 1)[1]
        definition = self.definitions[method]
        args = [x.arg for x in definition.args.args[1:]]
        supplied = dict(zip(args, call.args))
        if len(call.args) > len(args): raise ValueError("Extra operation arguments")
        for keyword in call.keywords:
            if keyword.arg not in args or keyword.arg in supplied: raise ValueError("Unexpected operation keyword")
            supplied[keyword.arg] = keyword.value
        defaults = dict(zip(args[len(args) - len(definition.args.defaults):], definition.args.defaults)) if definition.args.defaults else {}
        values = []
        for arg in args:
            if arg not in supplied and arg not in defaults: raise ValueError("Missing operation argument")
            code, kind = self.expression(supplied.get(arg, defaults.get(arg)))
            required = "timestamp" if method == "set_stamp_for_system" else "value"
            if kind != required: raise ValueError("Unexpected argument kind")
            values.append(code)
        result = f"actions.{method}(" + ", ".join(values) + ")"
        if self.native_balls:
            result="{ refresh_fixture_balls(&mut actions, &park, &["+", ".join(self.native_balls)+"]); "+result+" }"
        if simple_constructor(definition) is not None or method == "set_ball_harmonic": return result, "result"
        if method == "cloak_ball": return result, "result_bool"
        if method in {"get_pending_cloak_mode", "get_pending_interactive_state", "get_pending_speed_fraction"}: return result, "optional"
        if method in {"has_pending_cloak", "has_pending_uncloak"}: return result, "bool"
        if method in {"has_pending_set_free", "has_pending_set_not_free"}: return result, "value"
        if method in {"set_stamp_for_system", "undo_pending_cloak"}: return result, "unit"
        raise ValueError("Unreviewed non-constructor operation: " + method)

    def assertion(self, call):
        name = ast.unparse(call.func)
        if name == "self.assertSingleActionHistory" and len(call.args) == 2:
            ball, event = [self.expression(x)[0] for x in call.args]
            return f"assert!(actions.flush_history() == records(&[({ball}, INITIAL_TIMESTAMP, {event})]));"
        if name == "self.assertMultiActionHistory" and len(call.args) == 1:
            return f"assert!(actions.flush_history() == {self.expression(call.args[0])[0]});"
        if name == "self.assertHistoryEmpty" and not call.args:
            return "assert!(actions.flush_history().is_empty());"
        if name=="self.assertEqual" and len(call.args)==2 and not call.keywords:
            left,lk=self.expression(call.args[0]);right,rk=self.expression(call.args[1])
            if lk=="value" and rk=="optional": left=f"Some({left})"
            elif rk=="value" and lk=="optional": right=f"Some({right})"
            elif lk!=rk: raise ValueError("Unreviewed Python equality representation")
            return f"assert!({left} == {right});"
        if name == "self.assertActionInHistory" and len(call.args) == 2 and not call.keywords:
            ball, event = [self.expression(x)[0] for x in call.args]
            return f"assert!(contains_record(actions.flush_history(), {ball}, INITIAL_TIMESTAMP, {event}));"
        if name == "self.assertRaises" and len(call.args) >= 2 and isinstance(call.args[0], ast.Name) and call.args[0].id == "RuntimeError":
            operation = ast.Call(func=call.args[1], args=call.args[2:], keywords=call.keywords)
            code, kind = self.operation(operation)
            if kind != "result": raise ValueError("Unreviewed expected exception")
            return f"assert!({code} == Err(NetworkActionError::DynamicalOrientationDisabled));"
        if name in {"self.assertTrue", "self.assertFalse"} and len(call.args) == 1:
            code, kind = self.expression(call.args[0]); truth = name == "self.assertTrue"
            if kind == "result_bool": return f"assert!({code} == Ok({str(truth).lower()}));"
            if kind == "value": code += ".truthy()"
            elif kind != "bool": raise ValueError("Unreviewed truthiness")
            return f"assert!({'!' if not truth else ''}{code});"
        if name == "self.assertIsNone" and len(call.args) == 1:
            code, kind = self.expression(call.args[0])
            if kind != "optional": raise ValueError("Unreviewed None assertion")
            return f"assert!({code}.is_none());"
        raise ValueError("Unreviewed assertion helper: " + name)

    def translate(self, case, text):
        if not case.startswith(PREFIX): raise ValueError("Unknown qualified fixture")
        cls = next(x for x in ast.parse(text).body if isinstance(x, ast.ClassDef) and x.name == "TestActions")
        method = next(x for x in cls.body if isinstance(x, ast.FunctionDef) and x.name == case[len(PREFIX):])
        has_native=any(isinstance(n,ast.Call) and ast.unparse(n.func)=="add_ball_to_park" for n in ast.walk(method))
        lines = ["let mut actions = NetworkActions::new_for_empty_park_at_time_zero();"]
        if has_native:
            lines += ["let mut park=OriginalParkProjection::default();", "park.is_master=true;"]
        assertion_count = 0
        for node in method.body:
            if isinstance(node, ast.Assign) and len(node.targets) == 1:
                target = node.targets[0]
                if isinstance(target,ast.Name) and isinstance(node.value,ast.Call) and ast.unparse(node.value.func)=="add_ball_to_park":
                    lines+=self.native_creation(target.id,node.value)
                    continue
                if isinstance(target, ast.Tuple) and isinstance(node.value, ast.Tuple) and len(target.elts) == len(node.value.elts):
                    pairs = zip(target.elts, node.value.elts)
                else: pairs = [(target, node.value)]
                for name, value in pairs:
                    if not isinstance(name, ast.Name): raise ValueError("Unreviewed assignment destination")
                    code, kind = self.expression(value)
                    self.kinds[name.id] = kind
                    lines.append(f"let {name.id} = {code};")
            elif isinstance(node, ast.Expr) and isinstance(node.value, ast.Call):
                call = node.value; name = ast.unparse(call.func)
                if name in {"enable_dynamical_orientation", "disable_dynamical_orientation"} and not call.args and not call.keywords:
                    lines.append("actions.dynamical_orientation = " + ("true" if name.startswith("enable") else "false") + ";")
                elif name.startswith("self.assert"):
                    lines.append(f"// original-assertion: {TEST_SOURCE}:{node.lineno}")
                    lines.append(self.assertion(call)); assertion_count += 1
                else:
                    code, kind = self.operation(call)
                    if kind == "result": lines.append(f"assert!({code} == Ok(()));")
                    elif kind == "result_bool":
                        # Python deliberately discards this bool. Preserve its
                        # no-exception obligation without requiring True.
                        lines.append(f"assert!(matches!({code}, Ok(_))); // ignored original bool; no unexpected error")
                    elif kind == "unit": lines.append(code + ";")
                    else: raise ValueError("Unexpected ignored return value")
            else: raise ValueError("Unreviewed statement: " + ast.dump(node))
        if not assertion_count: raise ValueError("No original assertions translated")
        lines += ["// Original tearDown: ClearAll, release objects, reset settings.",
                  "actions.clear_park();", "actions.reset_settings_to_default();",
                  "#[cfg(kani)]", 'kani::cover!(true, "complete original empty-park action scenario");']
        if has_native:
            lines[-2:-2]=["assert!(!park.scope_error);", "park.clear_all();", "assert!(!park.scope_error);"]
            lines[-1]='kani::cover!(true, "original action assertions and native-fixture projection teardown completed");'
        attrs = "#[cfg_attr(kani, kani::proof)]\n#[cfg_attr(kani, kani::unwind(17))]\n#[cfg_attr(test, test)]\n"
        return f"// original-case: {case}\n// original-source-sha256: {hashlib.sha256(text.encode()).hexdigest()}\n" + attrs + f"fn {method.name.removeprefix('test_')}() {{\n" + "\n".join("    " + line for line in lines) + "\n}\n\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case")
    parser.add_argument("--prepare-model", action="store_true")
    parser.add_argument("--base", type=Path, default=BASE)
    parser.add_argument("--output-model", type=Path, default=MODEL_TARGET)
    parser.add_argument("--output-proof", type=Path, default=PROOF_TARGET)
    args = parser.parse_args()
    if args.prepare_model:
        prepare_model(args.output_model, args.base)
    if args.case:
        text = original(TEST_SOURCE)
        body = Translator(methods(original(MODEL_SOURCE))).translate(args.case, text)
        existing = args.output_proof.read_text() if args.output_proof.exists() else PREAMBLE
        if f"// original-case: {args.case}\n" in existing: raise ValueError("Case already translated")
        first_case=existing.find("// original-case: ")
        if first_case>=0: existing=PREAMBLE+existing[first_case:]
        args.output_proof.parent.mkdir(parents=True, exist_ok=True)
        args.output_proof.write_text(existing + body)
        print("Translated", args.case)
    if not args.case and not args.prepare_model: parser.error("Select a case or prepare the reviewed model")


if __name__ == "__main__": main()
