#!/usr/bin/env python3
"""Translate one reviewed concrete park scenario from the pinned Git AST.

Unknown calls, helpers, assertions and syntax fail before any proof is appended.
The state projection retains native boundary obligations. Invoke only with the
corresponding active Aegis packet; generator success is not proof evidence.
"""
import argparse
import ast
import hashlib
import math
import copy
from pathlib import Path
import subprocess

ROOT = Path.cwd()
REV = "ef738406b61ad06bae2f402857e464735ab106a2"
TARGET = ROOT / "verification/kani/src/park_contracts.rs"
SOURCES = {
    "python/destiny/test/ballpark/test_getters_and_setters.py",
    "python/destiny/test/ballpark/test_movement_controls.py",
    "python/destiny/test/ballpark/test_time.py",
    "python/destiny/test/ballpark/test_lifecycle_management.py",
    "python/destiny/test/test_ball.py",
    "python/destiny/test/ballpark/evolve/test_stop.py",
    "python/destiny/test/ballpark/evolve/test_goto.py",
    "python/destiny/test/ballpark/evolve/test_minis.py",
    "python/destiny/test/ballpark/evolve/test_simple_collision.py",
    "python/destiny/test/ballpark/evolve/test_follow.py",
    "python/destiny/test/ballpark/evolve/test_orbit.py",
    "python/destiny/test/ballpark/evolve/test_warp.py",
    "python/destiny/test/ballpark/evolve/test_missile.py",
    "python/destiny/test/ballpark/test_boxes_and_bubbles.py",
    "python/destiny/test/ballpark/test_callbacks.py",
    "python/destiny/test/ballpark/test_visibility.py",
}
FIELDS = {
    "mass": ("mass", "float"), "radius": ("radius", "f32"),
    "maxVelocity": ("max_velocity", "f32"), "Agility": ("agility", "f32"),
    "maxAngularVelocity":("max_angular_velocity","f32"),
    "RotationalAgility":("rotational_agility","f32"),
    "speedFraction": ("speed_fraction", "f32"),
    "isFree": ("is_free", "bool"), "isGlobal": ("is_global", "bool"),
    "isMassive": ("is_massive", "bool"), "isInteractive": ("is_interactive", "bool"),
    "isMoribund": ("is_moribund", "bool"), "isCloaked": ("is_cloaked", "i8"),
    "mode": ("mode", "mode"), "followId": ("follow_id", "int"),
    "followRange": ("follow_range", "f32"), "effectStamp": ("effect_stamp", "int"),
    "ownerId": ("owner_id", "int"), "harmonic": ("harmonic", "int"),
    "corporationID": ("corporation_id", "i32"), "allianceID": ("alliance_id", "i32"),
    "yaw": ("yaw", "float"), "pitch": ("pitch", "float"), "roll": ("roll", "float"),
    "newBubbleId": ("new_bubble", "int"), "oldBubbleId": ("old_bubble", "int"),
    **{key: ("position." + key, "float") for key in ("x", "y", "z")},
    **{key: ("velocity." + key[1:], "float") for key in ("vx", "vy", "vz")},
    **{key: ("goto." + key[-1].lower(), "float") for key in ("gotoX", "gotoY", "gotoZ")},
}
MODES = {"DSTBALL_" + name.upper(): "OriginalMode::" + name.title()
         for name in ("stop", "goto", "follow", "orbit", "missile", "warp", "formation", "boid", "troll", "rigid", "field", "mushroom")}
CALLS = {
    "SetBallMass": ("set_mass", ["int", "float"]),
    "SetBallRadius": ("set_radius", ["int", "f32"]),
    "SetMaxSpeed": ("set_max_speed", ["int", "f32"]),
    "SetBallAgility": ("set_agility_nonpositive_time_factor", ["int", "f32"]),
    "SetSpeedFraction": ("set_speed_fraction", ["int", "f32"]),
    "SetBallFree": ("set_free", ["int", "bool"]),
    "SetBallGlobal": ("set_global", ["int", "bool"]),
    "SetBallMassive": ("set_massive", ["int", "bool"]),
    "SetBallInteractive": ("set_interactive", ["int", "bool"]),
    "SetBoid": ("set_boid", ["int", "bool"]),
    "SetBallRigid": ("set_rigid", ["int"]),
    "SetBallTroll": ("set_troll", ["int", "i32"]),
    "SetBallHarmonic": ("set_harmonic", ["int", "int", "i32", "i32", "bool"]),
    "Stop": ("stop", ["int"]),
    "MissileFollow": ("missile_follow", ["int", "int", "int"]),
    "FormationFollow": ("formation_follow", ["int", "int", "i8"]),
    "SetBallFormation": ("set_formation", ["int", "i8"]),
    "RemoveBall": ("remove_ball", ["int", "i32"]),
    "Start": ("start", []), "Pause": ("pause", []),
    "AdjustTimes": ("adjust_times", ["int"]), "ClearAll": ("clear_all", []),
    "AddMushroom": ("add_mushroom", ["int","f32","float"]),
    "SetNotificationRange":("set_notification_range",["int","float"]),
    "SetTargetTracking":("set_target_tracking",["int","int","float"]),
    "RemoveProximitySensor":("remove_proximity_sensor",["int"]),
    "UncloakBall":("uncloak_ball",["int"]),
}


def original(path):
    return subprocess.run(["git", "show", f"{REV}:{path}"], cwd=ROOT, check=True,
                          capture_output=True, text=True).stdout


def convert(code, actual, expected):
    if expected == actual:
        return code
    if actual=="bool" and expected in {"int","i8","i32","float","f32"}:
        target={"int":"i64","i8":"i8","i32":"i32","float":"f64","f32":"f32"}[expected]
        return f"{target}::from(u8::from({code}))" if expected in {"float","f32"} else f"{target}::from({code})"
    if expected == "bool" and actual in {"int", "float", "f32", "i32", "i8"}:
        return f"({code}) != " + ("0.0" if actual in {"float", "f32"} else "0")
    if actual in {"int", "float", "f32", "i32", "i8"} and expected in {"int", "float", "f32", "i32", "i8"}:
        rust_type = {"int": "i64", "float": "f64", "f32": "f32", "i32": "i32", "i8": "i8"}[expected]
        return f"({code}) as {rust_type}"
    raise ValueError(f"Unreviewed conversion {actual} -> {expected}: {code}")


class Translation:
    def __init__(self, source):
        self.source = source
        self.env = {}
        self.balls = set()
        self.detached = set()
        self.lines = []
        self.helpers = {}
        self.indentation = 0
        self.maximum_iterations = 0
        self.point_capacity = 1
        self.point_variables = set()
        self.evolve_method = "evolve_single_body_linear"
        self.expected_exception = False
        self.mutable_detached=set()
        self.allow_canonical_boxes=False
        self.fixture_native_none=set()

    def emit(self, line):
        self.lines.append("    " + "    " * self.indentation + line)

    def expression(self, n):
        if isinstance(n, ast.Name) and n.id in self.env:
            return n.id, self.env[n.id]
        if isinstance(n,ast.Name) and n.id=="MIN_BOX_SIZE" and self.source.endswith("test_boxes_and_bubbles.py"):
            return "480.0_f64","float"
        if isinstance(n, ast.Constant):
            if n.value is None:
                return "None::<()>", "park-association"
            if isinstance(n.value, bool):
                return str(n.value).lower(), "bool"
            if isinstance(n.value, (int, float)):
                return repr(float(n.value)) + "_f64", "float"
        if isinstance(n, ast.UnaryOp) and isinstance(n.op, ast.USub):
            text, kind = self.expression(n.operand)
            return f"-({text})", kind
        if isinstance(n,ast.Compare) and len(n.ops)==1 and len(n.comparators)==1:
            operators={ast.Lt:"<",ast.LtE:"<=",ast.Gt:">",ast.GtE:">=",ast.Eq:"==",ast.NotEq:"!="}
            if type(n.ops[0]) not in operators:raise ValueError("Unreviewed comparison operator")
            return f"({self.typed(n.left,'float')} {operators[type(n.ops[0])]} {self.typed(n.comparators[0],'float')})","bool"
        if isinstance(n, ast.BinOp) and isinstance(n.op, (ast.Add, ast.Sub, ast.Mult, ast.Div)):
            left, lk = self.expression(n.left)
            right, rk = self.expression(n.right)
            left, right = convert(left, lk, "float"), convert(right, rk, "float")
            operator = {ast.Add: "+", ast.Sub: "-", ast.Mult:"*", ast.Div: "/"}[type(n.op)]
            return f"({left} {operator} {right})", "float"
        if isinstance(n, ast.Attribute):
            base = ast.unparse(n.value)
            if base == "self" and n.attr == "first_id_generated" and "first_id_generated" in self.env:
                return "first_id_generated", "int"
            if base=="self" and n.attr=="called" and self.source.endswith("test_callbacks.py"):
                return "park.ball_not_in_park_callback_called","bool"
            if base in self.env and self.env[base] == "mini-capsule":
                component = {"ax": "a.x", "ay": "a.y", "az": "a.z", "bx": "b.x", "by": "b.y", "bz": "b.z", "radius": "radius", "id": "id"}.get(n.attr)
                if component is None:
                    raise ValueError("Unreviewed capsule descriptor attribute")
                code = f"{base}.{component}"
                return (f"f64::from({code})" if n.attr == "radius" else code), "int" if n.attr == "id" else "float"
            if base in self.env and self.env[base] == "mini-box" and n.attr == "id":
                return f"{base}.id", "int"
            if base in self.env and self.env[base] == "mini-box":
                components={axis+str(i):field+"."+coordinate for axis,field in (("c","corner"),("x","x"),("y","y"),("z","z")) for i,coordinate in enumerate("xyz")}
                if n.attr not in components: raise ValueError("Unreviewed mini-box attribute")
                return f"{base}.{components[n.attr]}", "float"
            if base in self.env and self.env[base] == "mini-ball":
                if n.attr not in {"x","y","z","id","radius"}: raise ValueError("Unreviewed mini-ball attribute")
                if n.attr=="id": return f"{base}.id", "int"
                if n.attr=="radius": return f"f64::from({base}.radius)", "float"
                return f"{base}.position.{n.attr}", "float"
            if base == "destiny" and n.attr in MODES:
                return MODES[n.attr], "mode"
            if base=="destiny" and n.attr=="DSTNORMALCLOAK":return "1_i8","i8"
            if base=="math" and n.attr=="pi":return "core::f64::consts::PI","float"
            if base=="helpers" and n.attr=="AU":return "149_597_870_700.0_f64","float"
            if base in self.balls:
                ball = base if base in self.detached else f"park.retained({base})"
                if n.attr == "id":
                    return f"{ball}.id" if base in self.detached else base, "int"
                if n.attr == "formationID":
                    return f"i64::from({ball}.formation_id as u8)", "int"
                if n.attr == "ballpark":
                    return f"{ball}.in_park.then_some(())", "park-association"
                if n.attr in {"range_callback_args","target_tracking_callback_args"}:
                    return f"{ball}.{n.attr}","ids"
                if n.attr=="proximity_callback_args":return f"{ball}.{n.attr}","proximity-events"
                if n.attr in {"miniBalls", "miniCapsules", "miniBoxes"}:
                    return ball + "." + {"miniBalls": "mini_balls", "miniCapsules": "mini_capsules", "miniBoxes": "mini_boxes"}[n.attr], "child-list:" + n.attr
                if n.attr in FIELDS:
                    field, kind = FIELDS[n.attr]
                    code = f"{ball}.{field}"
                    if kind == "f32":
                        return f"f64::from({code})", "float"
                    if kind == "i32":
                        return f"i64::from({code})", "int"
                    return code, kind
            if base == "self.park" and n.attr in {"time", "isRunning"}:
                return "park." + {"time": "time", "isRunning": "is_running"}[n.attr], "int" if n.attr == "time" else "bool"
            if base == "self.park" and n.attr=="balls":
                return "park", "ball-map"
            if base == "helpers" and n.attr == "TEN_BILLION":
                return "1e10_f64", "float"
        if isinstance(n, ast.Tuple):
            parts = [self.expression(x) for x in n.elts]
            return "(" + ", ".join(x[0] for x in parts) + ")", "tuple:" + ",".join(x[1] for x in parts)
        if isinstance(n, ast.List):
            if n.elts and all(isinstance(x,ast.Tuple) and len(x.elts)==2 for x in n.elts):
                entries=["("+self.typed(x.elts[0],"float")+", "+self.typed(x.elts[1],"int")+")" for x in n.elts]
                return "distance_list(&["+", ".join(entries)+"])", "distances"
            if n.elts and all(isinstance(x,ast.Tuple) and len(x.elts)==3 for x in n.elts):
                return f"point_list::<{self.point_capacity}>(&[" + ", ".join(self.vec(x.elts) for x in n.elts) + "])", f"points:{self.point_capacity}"
            parts = [convert(*self.expression(x), "int") for x in n.elts]
            return "id_list(&[" + ", ".join(parts) + "])", "ids"
        if isinstance(n,ast.Dict):
            if n.keys and all(isinstance(key,ast.Constant) and isinstance(key.value,str) for key in n.keys):
                entries=[]
                for key,value in zip(n.keys,n.values):
                    if key.value!="foo":raise ValueError("Unreviewed original object dictionary key")
                    code,kind=self.expression(value)
                    if kind=="float":payload="ActionValue::Float("+code+")"
                    elif kind in {"int","i32","i8"}:payload="ActionValue::Integer("+convert(code,kind,"int")+")"
                    elif kind=="bool":payload="ActionValue::Boolean("+code+")"
                    else:raise ValueError("Unreviewed original dictionary value")
                    entries.append("(OriginalBoxObjectKey::Foo,"+payload+")")
                return "box_object(&["+", ".join(entries)+"])","object-dictionary"
            entries=[]
            for key,value in zip(n.keys,n.values):
                if key is None: raise ValueError("Unreviewed dictionary expansion")
                if isinstance(value,ast.List):
                    payload="OriginalBubbleDelta::Balls(id_list(&["+", ".join(self.typed(x,"int") for x in value.elts)+"]))"
                else:payload="OriginalBubbleDelta::Bubble("+self.typed(value,"int")+")"
                entries.append("("+self.typed(key,"int")+","+payload+")")
            return "delta_map(&["+", ".join(entries)+"])","delta-map"
        if isinstance(n, ast.Subscript):
            if ast.unparse(n.value)=="self.park.balls":
                return self.typed(n.slice,"int"), "existing-ball-id"
            code, kind = self.expression(n.value)
            index=ast.literal_eval(n.slice)
            if kind=="point" and type(index)is int and 0<=index<3:
                return code+"."+"xyz"[index],"float"
            if kind not in {"child-list:miniCapsules", "child-list:miniBoxes", "child-list:miniBalls"} or index not in (0,-1):
                raise ValueError("Unreviewed child descriptor index")
            actual_index="0" if index==0 else f"({code}).len().saturating_sub(1)"
            return f"({code}).get({actual_index})", {"child-list:miniCapsules":"capsule-option", "child-list:miniBoxes":"box-option", "child-list:miniBalls":"mini-ball-option"}[kind]
        if isinstance(n, ast.Call):
            method = ast.unparse(n.func)
            if method=="set" and len(n.args)==1 and not n.keywords:
                code,kind=self.expression(n.args[0])
                if kind.startswith("points:"):return code,"point-set:"+kind.split(":")[1]
                if kind!="ids": raise ValueError("Unreviewed set representation")
                return code,"id-set"
            if method=="sorted" and len(n.args)==1 and not n.keywords:
                code,kind=self.expression(n.args[0])
                if not kind.startswith("points:"):raise ValueError("Unreviewed sorted representation")
                return f"sort_points(&{code})",kind
            if method in self.helpers:
                helper = self.helpers[method]
                parameters=[arg.arg for arg in helper.args.args[1:]]
                if n.keywords or helper.args.defaults or len(n.args)!=len(parameters):
                    raise ValueError("Unreviewed helper arguments")
                if helper.decorator_list or not isinstance(helper.body[-1], ast.Return):
                    raise ValueError("Unreviewed helper control flow")
                replacements=dict(zip(parameters,n.args))
                class BindArguments(ast.NodeTransformer):
                    def visit_Name(self, node):
                        return copy.deepcopy(replacements.get(node.id,node))
                translated=[BindArguments().visit(copy.deepcopy(statement)) for statement in helper.body]
                for statement in translated[:-1]:
                    self.statement(statement)
                return self.expression(translated[-1].value)
            if method=="math.radians" and len(n.args)==1 and not n.keywords:
                degrees=ast.literal_eval(n.args[0])
                if type(degrees) not in (float,int) or not math.isfinite(degrees):
                    raise ValueError("Unreviewed Python radians input")
                return repr(math.radians(degrees))+"_f64","float"
            if method == "float" and len(n.args) == 1 and ast.literal_eval(n.args[0]) == "nan":
                return "f64::NAN", "float"
            if method == "len" and len(n.args) == 1:
                if ast.unparse(n.args[0]) == "helpers.FORMATIONS":
                    return "3_i64", "int"
                if ast.unparse(n.args[0]) == "self.park.balls":
                    return "park.count() as i64", "int"
                code, kind = self.expression(n.args[0])
                if kind != "ids" and not kind.startswith("child-list:"):
                    raise ValueError("Unreviewed len target")
                return f"({code}).len() as i64", "int"
            if method == "helpers.reinterpret_double_as_int" and len(n.args) == 1:
                code = convert(*self.expression(n.args[0]), "float")
                return f"({code}).to_bits() as i64", "int"
            if method.startswith("self.park."):
                return self.park_call(n)
            if isinstance(n.func, ast.Attribute) and ast.unparse(n.func.value) in self.balls:
                parent = ast.unparse(n.func.value)
                if n.func.attr=="ReserveFormationSlot" and not n.args and not n.keywords:
                    if parent in self.detached:
                        return "-1_i8","i8"  # source unassigned/detached guard
                    return f"park.reserve_original_formation_slot({parent})","i8"
                if parent in self.detached:
                    if n.func.attr=="AddMiniBall" and len(n.args)==4 and not n.keywords:
                        return "park.add_detached_mini_ball(&mut "+parent+", "+self.vec(n.args[:3])+", "+self.typed(n.args[3],"f32")+")","park-result"
                    raise ValueError("Unreviewed detached child operation")
                name = n.func.attr
                if n.keywords:
                    raise ValueError("Unreviewed child operation keywords")
                if name == "AddMiniBall" and len(n.args) == 4:
                    return f"park.add_mini_ball({parent}, {self.vec(n.args[:3])}, {self.typed(n.args[3], 'f32')})", "park-result"
                if name == "AddMiniCapsule" and len(n.args) == 7:
                    return f"park.add_mini_capsule({parent}, {self.vec(n.args[:3])}, {self.vec(n.args[3:6])}, {self.typed(n.args[6], 'f32')})", "park-result"
                if name == "AddMiniBox" and len(n.args) == 12:
                    return f"park.add_mini_box({parent}, " + ", ".join(self.vec(n.args[i:i+3]) for i in range(0,12,3)) + ")", "park-result"
                if name == "AddProximitySensor" and len(n.args) == 4:
                    self.emit("// External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.")
                    self.emit("#[cfg(kani)] let random_draw: u32 = kani::any();")
                    self.emit("#[cfg(kani)] kani::assume(random_draw <= 2_147_483_647);")
                    self.emit("#[cfg(test)] let random_draw: u32 = 0;")
                    return f"park.add_proximity_sensor({parent}, " + ", ".join(self.typed(arg, kind) for arg,kind in zip(n.args,["f32","float","i32","bool"])) + ", random_draw, 2_147_483_647)", "park-result"
        raise ValueError("Unreviewed expression: " + ast.unparse(n))

    def typed(self, n, kind):
        return convert(*self.expression(n), kind)

    def vec(self, nodes):
        if len(nodes) != 3:
            raise ValueError("Expected all three coordinates")
        return "Vec3 { " + ", ".join(f"{axis}: {self.typed(n, 'float')}" for axis, n in zip("xyz", nodes)) + " }"

    def park_call(self, n):
        name = n.func.attr
        if n.keywords:
            raise ValueError("Unreviewed park keywords")
        args = list(n.args)
        if name=="LaunchMissile" and len(args)==5:
            return "park.launch_missile("+", ".join(self.typed(arg,kind) for arg,kind in zip(args,("int","int","int","i8","i8")))+")","park-result"
        if name=="EntityWarpIn" and len(args)==5:
            return f"park.entity_warp_in({self.typed(args[0],'int')},{self.vec(args[1:4])},{self.typed(args[4],'i32')})","park-result"
        if name=="SetBoxObject" and len(args)==4:
            object,kind=self.expression(args[-1])
            if kind!="object-dictionary":raise ValueError("Original primitive dictionary object required")
            temporary="object_set_"+str(len(self.lines))
            self.emit(f"let {temporary}=park.set_box_object({self.vec(args[:3])},{object});")
            self.emit(f"assert!({temporary}.is_ok()); // original object setter must not raise")
            return f"{temporary}.unwrap_or(false)","bool"
        if name=="GetBoxObject" and len(args)==3:
            temporary="object_get_"+str(len(self.lines))
            self.emit(f"let {temporary}=park.get_box_object({self.vec(args)});")
            self.emit(f"assert!({temporary}.is_ok()); // original object getter must not raise")
            return f"{temporary}.unwrap_or(None)","optional-object"
        if name in {"GetBallBox","GetStaticCollidableBox"} and len(args)==1:
            self.maximum_iterations=max(self.maximum_iterations,128)
            temporary="box_result_"+str(len(self.lines))
            operation="get_ball_box" if name=="GetBallBox" else "get_static_collidable_box"
            self.emit(f"let {temporary}=park.{operation}({self.typed(args[0],'int')});")
            self.emit(f"assert!({temporary}.is_ok()); // original box query must not raise")
            self.emit(f"let {temporary}={temporary}.unwrap_or(None);")
            self.emit(f"assert!({temporary}.is_some()); // original tuple unpacking requires a box")
            return f"{temporary}.unwrap_or((0.0,(0.0,0.0,0.0)))","box-snapshot"
        if name=="GetBoxCenter" and len(args)==4:
            temporary="center_result_"+str(len(self.lines))
            self.emit(f"let {temporary}=park.get_grid_center({self.typed(args[0],'i32')},{self.vec(args[1:])});")
            self.emit(f"assert!({temporary}.is_ok()); // source level/coordinate conversion must be defined")
            return f"{temporary}.unwrap_or((0.0,0.0,0.0))","tuple:float,float,float"
        if name=="GetActiveBoxes" and len(args)==1:
            self.maximum_iterations=max(self.maximum_iterations,128)
            temporary="boxes_result_"+str(len(self.lines))
            self.emit(f"let {temporary}=park.get_active_box_view({self.typed(args[0],'i32')},{str(self.allow_canonical_boxes).lower()});")
            self.emit(f"assert!({temporary}.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view")
            return f"{temporary}.unwrap_or(None)","optional-boxes"
        if name=="CheckVisibility" and len(args)==2:
            return "park.check_visibility("+", ".join(self.typed(n,"int") for n in args)+")","int"
        if name=="CloakBall" and len(args) in (2,3):
            if len(args)==2:args.append(ast.Constant(value=2000.0))
            return "park.cloak_ball("+", ".join(self.typed(n,k) for n,k in zip(args,("int","i8","f32")))+")","park-result"
        if name=="ScanCone" and len(args)==6:
            temporary="scan_result_"+str(len(self.lines))
            code="park.scan_cone("+", ".join(self.typed(n,k) for n,k in zip(args[:3],("int","float","float")))+", "+self.vec(args[3:])+")"
            self.emit(f"let {temporary}={code};")
            self.emit(f"assert!({temporary}.is_ok()); // original scan must not raise")
            self.emit(f"let {temporary}={temporary}.unwrap_or(None);")
            self.emit(f"assert!({temporary}.is_some()); // original result is a list, not None")
            return f"{temporary}.unwrap_or_default()","ids"
        if name=="SetBallNotInParkCallback" and len(args)==1:
            if ast.unparse(args[0])=="self.handle_ball_not_in_park_error":enabled="true"
            elif isinstance(args[0],ast.Constant) and args[0].value is None:enabled="false"
            else:raise ValueError("Unreviewed Python callback contract")
            return "park.set_ball_not_in_park_callback("+enabled+")","unit"
        if name=="InitializeBubbles" and len(args)<=1:
            self.maximum_iterations=max(self.maximum_iterations,64)
            src=self.typed(args[0],"int") if args else "0_i64"
            return f"park.initialize_query_bubbles({src})", "park-result"
        if name=="CopyBubbles" and not args:
            return "park.copy_query_bubbles()","park-result"
        if name=="AdditionsAndDeletions" and len(args)==5:
            outputs=[]
            for n in args[:4]:
                code,kind=self.expression(n)
                if not isinstance(n,ast.Name) or kind!="delta-map":raise ValueError("Original mutable dictionary required")
                outputs.append(code)
            users,kind=self.expression(args[4])
            if kind!="user-ids":raise ValueError("Original user ship list required")
            code=f"park.bubble_additions_and_deletions(&{users})"
            if self.expected_exception:return code,"result-updates"
            temporary="bubble_update_result_"+str(len(self.lines))
            self.emit(f"let {temporary}={code};")
            self.emit(f"assert!({temporary}.is_ok()); // original native mutation must not raise")
            self.emit(f"let {temporary}={temporary}.unwrap_or_default();")
            for target,field in zip(outputs,("additions_per_player","deletions_per_player","additions_per_bubble","deletions_per_bubble")):
                self.emit(f"store_delta_map(&mut {target},{temporary}.{field});")
            return "()","unit"
        if name in {"GetBallIdsInRange","GetBallIdsAndDistInRange"} and len(args) in (2,3,4,5):
            if len(args) in (2,3):
                src="Some("+self.typed(args[0],"int")+")";position="Vec3::ZERO"; radius=self.typed(args[1],"float")
                cloaked=self.typed(args[2],"bool") if len(args)==3 else "false"
            else:
                src="None";position=self.vec(args[:3]);radius=self.typed(args[3],"float")
                cloaked=self.typed(args[4],"bool") if len(args)==5 else "false"
            code=f"park.query_range({src}, {position}, {radius}, {cloaked})"
            if self.expected_exception: return code,"result-distances"
            self.maximum_iterations=max(self.maximum_iterations,64)
            tmp="query_result_"+str(len(self.lines))
            self.emit(f"let {tmp}={code};")
            self.emit(f"assert!({tmp}.is_ok()); // original query must not raise")
            value=f"{tmp}.unwrap_or_default()"
            return (value,"distances") if name=="GetBallIdsAndDistInRange" else (f"query_ids(&{value})","ids")
        if name in {"GetBallIdsInRangeOfTriangle","GetBallIdsInCapsule","GetBallIdsInCone"}:
            count=8 if name=="GetBallIdsInRangeOfTriangle" else 5
            if len(args)!=count: raise ValueError("Unreviewed spatial query arity")
            operation={"GetBallIdsInRangeOfTriangle":"query_triangle","GetBallIdsInCapsule":"query_capsule","GetBallIdsInCone":"query_cone"}[name]
            code="park."+operation+"("+self.typed(args[0],"int")+", "+self.vec(args[1:4])+", "
            if count==8: code+=self.vec(args[4:7])+", "
            code+=self.typed(args[-1],"float")+")"
            if self.expected_exception: return code,"result-ids"
            self.maximum_iterations=max(self.maximum_iterations,64)
            tmp="query_result_"+str(len(self.lines))
            self.emit(f"let {tmp}={code};")
            self.emit(f"assert!({tmp}.is_ok()); // original query must not raise")
            return f"{tmp}.unwrap_or_default()","ids"
        if name=="GetBubbleAtCoordinates" and len(args)==3:
            tmp="query_result_"+str(len(self.lines))
            self.emit(f"let {tmp}=park.query_bubble_at({self.vec(args)});")
            self.emit(f"assert!({tmp}.is_ok()); // original coordinate lookup must not raise")
            return f"{tmp}.unwrap_or(None)","optional-id"
        if name == "AddBall" and len(args) == 17:
            scalar = [(0,"id","int"),(1,"mass","float"),(2,"radius","f32"),(3,"max_velocity","f32"),
                      (4,"is_free","bool"),(5,"is_global","bool"),(6,"is_massive","bool"),
                      (7,"is_interactive","bool"),(8,"is_space_junk","bool"),(15,"agility","f32"),(16,"speed_fraction","f32")]
            fields = [f"{field}: {self.typed(args[i], kind)}" for i,field,kind in scalar]
            fields += ["position: " + self.vec(args[9:12]), "velocity: " + self.vec(args[12:15])]
            return "park.add_ball(OriginalBallInit { " + ", ".join(fields) + " })", "new-ball-id"
        if name in {"SetBallPosition", "SetBallVelocity", "GotoPoint", "GotoDirection"}:
            if len(args) != 4:
                raise ValueError("Unexpected vector operation arity")
            method = {"SetBallPosition": "set_position", "SetBallVelocity": "set_velocity",
                      "GotoPoint": "goto_point", "GotoDirection": "goto_direction"}[name]
            return f"park.{method}({self.typed(args[0], 'int')}, {self.vec(args[1:])})", "unit"
        if name in {"FollowBall", "Orbit"}:
            if len(args) == 2:
                args.append(ast.Constant(value=1.0))
            if len(args) != 3:
                raise ValueError("Unexpected follow/orbit arity")
            return "park.follow(" + ", ".join(self.typed(n, k) for n, k in zip(args, ["int", "int", "f32"])) + f", {str(name == 'Orbit').lower()})", "unit"
        if name == "WarpTo":
            if not 4 <= len(args) <= 6:
                raise ValueError("Unexpected WarpTo arity")
            args += [ast.Constant(value=v) for v in [20000.0, 20][len(args)-4:]]
            return f"park.warp_to({self.typed(args[0], 'int')}, {self.vec(args[1:4])}, {self.typed(args[4], 'float')}, {self.typed(args[5], 'i32')})", "unit"
        if name == "LoadFormations" and len(args) == 1 and ast.unparse(args[0]) == "helpers.FORMATIONS":
            return "park.load_original_formations()", "unit"
        if name == "GetFollowers" and len(args) == 1:
            return f"park.followers({self.typed(args[0], 'int')})", "ids"
        if name=="GetRemoteFollowers" and not args:
            temporary="remote_followers_result_"+str(len(self.lines))
            self.emit(f"let {temporary}=park.remote_followers();")
            self.emit(f"assert!({temporary}.is_ok()); // original remote follower call must not raise")
            return f"{temporary}.unwrap_or_default()","ids"
        if name in {"GetCenterDist", "GetSurfaceDist"} and len(args) == 2:
            return "park." + {"GetCenterDist": "center_distance", "GetSurfaceDist": "surface_distance"}[name] + "(" + ", ".join(self.typed(n, "int") for n in args) + ")", "float"
        if name == "GetAccuracy" and len(args) == 7:
            return "park.accuracy(" + ", ".join(self.typed(n, k) for n, k in zip(args, ["int", "int", "float", "float", "float", "float", "float"])) + ")", "tuple:float,float,float"
        if name == "GetCurrentEgoPos" and not args:
            # These cases construct no ego; GetReferencePoint returns the zero vector.
            return "park.current_ego_position_without_ego()", "tuple:float,float,float"
        if name == "Evolve" and not args:
            return f"park.{self.evolve_method}()", "park-result"
        if name == "RemoveBall" and len(args) == 1:
            args.append(ast.Constant(value=0))
        if name in CALLS:
            method, kinds = CALLS[name]
            if len(args) != len(kinds):
                raise ValueError("Unexpected arity: " + name)
            if name == "SetBallMass":
                try:
                    if ast.literal_eval(args[1]) <= 0.0:
                        method = "reject_nonpositive_mass"
                except (ValueError, TypeError):
                    pass
            return "park." + method + "(" + ", ".join(self.typed(n, k) for n, k in zip(args, kinds)) + ")", "unit"
        raise ValueError("Unimplemented original park operation: " + name)

    def add_ball(self, name, keywords):
        if name == "_":
            name = "ignored_ball_" + str(len(self.balls))
        simple = {"objectID": ("id", "int"), "mass": ("mass", "float"),
                  "radius": ("radius", "f32"), "maxVel": ("max_velocity", "f32"),
                  "agility": ("agility", "f32"), "speedFraction": ("speed_fraction", "f32"),
                  **{x: (y, "bool") for x, y in [("isFree", "is_free"), ("isMassive", "is_massive"), ("isGlobal", "is_global"), ("isInteractive", "is_interactive")]}}
        fields, pos, vel = [], {}, {}
        for key, value in keywords.items():
            if key in simple:
                field, kind = simple[key]
                fields.append(field + ": " + self.typed(value, kind))
            elif key in {"x", "y", "z"}:
                pos[key] = value
            elif key in {"vx", "vy", "vz"}:
                vel[key[1:]] = value
            else:
                raise ValueError("Unreviewed fixture keyword: " + key)
        if pos:
            fields.append("position: " + self.vec([pos.get(k, ast.Constant(value=1e10)) for k in "xyz"]))
        if vel:
            fields.append("velocity: " + self.vec([vel.get(k, ast.Constant(value=0.0)) for k in "xyz"]))
        constructor = "add_nonpositive_time_factor_ball"
        if "mass" in keywords and ast.literal_eval(keywords["mass"]) > 0.0:
            constructor = "add_ball"
        self.emit(f"let {name} = park.{constructor}(OriginalBallInit {{ " + ", ".join(fields + ["..OriginalBallInit::default()"]) + " });")
        self.emit(f"assert!(park.contains({name}) && !park.scope_error);")
        self.env[name] = "int"
        self.balls.add(name)

    def assignment(self, target, value):
        if ast.unparse(target) in self.fixture_native_none:
            code,kind=self.expression(value)
            if kind!="park-result":raise ValueError("Original native fixture call must return None")
            self.emit(f"assert!({code}==Ok(())); // original fixture call must not raise")
            self.emit("let _original_fixture_"+target.attr+"_return=(); // preserve the original Python None member")
            return
        if ast.unparse(target)=="helpers.create_space_ball.next_object_id":
            counter_value=ast.literal_eval(value)
            if type(counter_value)is not int or not -(1<<63)<=counter_value<(1<<63):raise ValueError("Helper counter reset outside int64 adapter scope")
            self.emit(f"next_space_object_id={counter_value}_i64; // original global helper reset; preserve its exact value")
            return
        if isinstance(target, ast.Name):
            if target.id=="user_ships" and isinstance(value,ast.List):
                values=["None::<i64>" if isinstance(n,ast.Constant) and n.value is None else "Some("+self.typed(n,"int")+")" for n in value.elts]
                self.env[target.id]="user-ids"
                self.emit(f"let {target.id}=user_list(&["+", ".join(values)+"]);")
                return
            if target.id=="expected_transitions" and isinstance(value,ast.List):
                entries=[]
                for n in value.elts:
                    if not isinstance(n,ast.Tuple) or len(n.elts)!=3:raise ValueError("Original transition tuple required")
                    entries.append("("+", ".join(self.typed(x,"int") for x in n.elts)+")")
                self.env[target.id]="transitions"
                self.emit(f"let {target.id}=transition_list(&["+", ".join(entries)+"]);")
                return
            if target.id in self.point_variables and isinstance(value,ast.List) and not value.elts:
                self.env[target.id]=f"points:{self.point_capacity}"
                self.emit(f"let mut {target.id}=BoundedList::<Vec3,{self.point_capacity}>::new();")
                return
            if isinstance(value, ast.Call) and ast.unparse(value.func) == "helpers.create_space_ball":
                if len(value.args) != 1 or ast.unparse(value.args[0]) != "self.park":
                    raise ValueError("Unreviewed space-ball fixture target")
                keywords = {k.arg:k.value for k in value.keywords}
                if len(keywords)!=len(value.keywords) or set(keywords)-set(("x","y","z","vx","vy","vz","mass","max_velocity")):
                    raise ValueError("Unreviewed space-ball keywords")
                creation = {k:v for k,v in keywords.items() if k in {"x","y","z","vx","vy","vz"}}
                creation["objectID"] = ast.Name(id="next_space_object_id")
                for axis in ("x","y","z"):
                    creation.setdefault(axis,ast.Constant(value=0))
                self.add_ball(target.id,creation)
                self.emit("let next_id = next_space_object_id.checked_add(1);")
                self.emit("assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted")
                self.emit("next_space_object_id = next_id.unwrap_or(next_space_object_id);")
                self.emit(f"park.set_free({target.id}, true);")
                for attribute,argument,default in [("maxVelocity","max_velocity",10.0),("Agility",None,0.9),
                        ("mass","mass",13_000_000.0),("radius",None,2.0),("isMassive",None,True),("speedFraction",None,0.95)]:
                    requested=keywords.get(argument) if argument else None
                    if requested is None or isinstance(requested,ast.Constant) and requested.value is None:
                        requested=ast.Constant(value=default)
                    self.assignment(ast.Attribute(value=ast.Name(id=target.id),attr=attribute),requested)
                return
            if isinstance(value, ast.Call) and ast.unparse(value.func) == "destiny.Ball":
                if value.args or value.keywords:
                    raise ValueError("Unreviewed detached constructor arguments")
                self.emit(f"let {'mut ' if target.id in self.mutable_detached else ''}{target.id} = OriginalParkBall::default();")
                self.env[target.id] = "ball"
                self.balls.add(target.id)
                self.detached.add(target.id)
                return
            if isinstance(value, ast.Call) and ast.unparse(value.func) == "helpers.add_ball_to_park":
                if len(value.args) != 1 or ast.unparse(value.args[0]) != "self.park":
                    raise ValueError("Unreviewed fixture target")
                self.add_ball(target.id, {k.arg: k.value for k in value.keywords})
                return
            if isinstance(value,ast.Call) and ast.unparse(value.func)=="helpers.add_ball_event_spy_to_park":
                if len(value.args)!=1 or ast.unparse(value.args[0])!="self.park":raise ValueError("Unreviewed event spy constructor")
                self.add_ball(target.id,{k.arg:k.value for k in value.keywords})
                self.emit(f"park.write_attribute({target.id},|b|b.event_spy=true); // original BallEventSpy wraps this same native ball and initializes its argument lists")
                return
            code, kind = self.expression(value)
            if kind in {"capsule-option", "box-option", "mini-ball-option"}:
                temporary = f"child_access_{len(self.lines)}"
                self.emit(f"let {temporary} = {code};")
                self.emit(f"assert!({temporary}.is_some()); // original list index must exist")
                fallback = {"capsule-option":"OriginalMiniCapsule { id: 0, parent_id: 0, a: Vec3::ZERO, b: Vec3::ZERO, radius: 0.0 }", "box-option":"OriginalMiniBox { id: 0, parent_id: 0, corner: Vec3::ZERO, x: Vec3::ZERO, y: Vec3::ZERO, z: Vec3::ZERO }", "mini-ball-option":"OriginalMiniBall { id: 0, position: Vec3::ZERO, radius: 0.0 }"}[kind]
                self.emit(f"let {target.id} = {temporary}.unwrap_or({fallback});")
                self.env[target.id] = {"capsule-option":"mini-capsule", "box-option":"mini-box", "mini-ball-option":"mini-ball"}[kind]
                return
            if kind in {"new-ball-id", "existing-ball-id"}:
                self.emit(f"let {target.id} = {code};")
                self.emit(f"assert!(park.contains({target.id}) && !park.scope_error);")
                self.env[target.id] = "int"
                self.balls.add(target.id)
                return
            self.env[target.id] = kind
            self.emit(f"let {'mut ' if kind=='delta-map' else ''}{target.id} = {code};")
            return
        if isinstance(target, ast.Tuple):
            if isinstance(value, ast.Call) and ast.unparse(value.func) == "self.add_balls":
                count = ast.literal_eval(value.args[0])
                if len(value.args) != 1 or count != len(target.elts) or not 1 <= count <= 8:
                    raise ValueError("Incomplete fixture loop translation")
                for i, t in enumerate(target.elts):
                    if not isinstance(t, ast.Name):
                        raise ValueError("Unexpected fixture binding")
                    self.add_ball(t.id, {"objectID": ast.Constant(value=i+1)})
                return
            if isinstance(value, ast.Tuple):
                if len(target.elts) != len(value.elts):
                    raise ValueError("Tuple assignment mismatch")
                for t, v in zip(target.elts, value.elts):
                    self.assignment(t, v)
                return
            code, kind = self.expression(value)
            if kind=="optional-boxes" and len(target.elts)==2 and all(isinstance(n,ast.Name) for n in target.elts):
                temporary="box_unpack_"+str(len(self.lines))
                self.emit(f"let {temporary}={code};");self.emit(f"assert!({temporary}.is_some()); // original tuple unpacking requires a value")
                self.env[target.elts[0].id]="float";self.env[target.elts[1].id]="points:64"
                self.emit("let ("+", ".join(n.id for n in target.elts)+f")={temporary}.unwrap_or((0.0,BoundedList::<Vec3,64>::new()));")
                return
            if kind=="box-snapshot" and len(target.elts)==2 and isinstance(target.elts[0],ast.Name) and isinstance(target.elts[1],ast.Tuple) and len(target.elts[1].elts)==3 and all(isinstance(n,ast.Name) for n in target.elts[1].elts):
                names=[target.elts[0].id]+[n.id for n in target.elts[1].elts]
                self.env.update({name:"float" for name in names})
                self.emit(f"let ({names[0]},({','.join(names[1:])}))={code};")
                return
            kinds = kind.removeprefix("tuple:").split(",")
            if not kind.startswith("tuple:") or len(kinds) != len(target.elts) or any(not isinstance(t, ast.Name) for t in target.elts):
                raise ValueError("Unreviewed destructuring")
            names = [t.id for t in target.elts]
            self.env.update(zip(names, kinds))
            self.emit("let (" + ", ".join(names) + ") = " + code + ";")
            return
        if isinstance(target, ast.Attribute) and isinstance(target.value, ast.Name) and target.value.id in self.balls and target.attr in FIELDS:
            field, kind = FIELDS[target.attr]
            # Python evaluates the right-hand side before assigning the member.
            # Keep that order and avoid capturing an immutable park borrow in
            # the closure that mutates the selected ball.
            temporary = f"attribute_value_{len(self.lines)}"
            self.emit(f"let {temporary} = {self.typed(value, kind)};")
            notification = {"mass": "Mass", "radius": "Radius", "maxVelocity": "MaxSpeed",
                "maxAngularVelocity":"MaxAngularSpeed","RotationalAgility":"RotationalAgility",
                "Agility": "Agility", "speedFraction": "SpeedFraction", "isFree": "Free",
                "isMassive": "Massive", "isGlobal": "Global", "harmonic": "Harmonic",
                "corporationID": "Corporation", "allianceID": "Alliance"}.get(target.attr, "NoHandler")
            if field.startswith("position."): notification = "Position"
            elif field.startswith("velocity."): notification = "Velocity"
            self.emit(f"park.write_attribute_notified({target.value.id}, |b| b.{field} = {temporary}, OriginalBallAttribute::{notification});")
            return
        raise ValueError("Unreviewed assignment: " + ast.unparse(target))

    def statement(self, n):
        if isinstance(n, ast.Assign) and len(n.targets)==1 and ast.unparse(n.targets[0])=="self.maxDiff" and isinstance(n.value,ast.Constant) and n.value.value is None:
            self.emit("// unittest maxDiff=None changes failure diagnostics, not simulation state.")
            return
        if isinstance(n, ast.Assign) and len(n.targets) == 1:
            return self.assignment(n.targets[0], n.value)
        if isinstance(n, ast.AugAssign) and isinstance(n.op, ast.Add):
            return self.assignment(n.target, ast.BinOp(left=n.target, op=n.op, right=n.value))
        if isinstance(n,ast.AugAssign) and isinstance(n.op,ast.Sub):
            return self.assignment(n.target,ast.BinOp(left=n.target,op=n.op,right=n.value))
        if isinstance(n,ast.Try):
            # The exact pinned Ball_Blue maps both fields READWRITE|NOTIFY.
            # This original fallback supports older modules; the selected pin
            # takes the accepted assignment path. Python dispatch remains a
            # separately required executable/native correspondence obligation.
            expected=ast.parse("try:\n fixture_missile.maxAngularVelocity=2.0\n fixture_missile.RotationalAgility=0.5\nexcept AttributeError:\n pass\n").body[0]
            if ast.dump(n)!=ast.dump(expected):raise ValueError("Unreviewed try/except or optional property dependency")
            self.emit("// Pinned Ball_Blue.cpp exposes both rotational fields; retain its accepted path.")
            for statement in n.body:self.statement(statement)
            return
        if isinstance(n,ast.For) and isinstance(n.target,ast.Name) and not n.orelse:
            if isinstance(n.iter,ast.Name) and self.env.get(n.iter.id,"").startswith("points:"):
                self.env[n.target.id]="point"
                counter=f"point_iteration_{len(self.lines)}";temporary=f"point_access_{len(self.lines)}"
                self.emit(f"let mut {counter}=0_usize;")
                self.emit(f"while {counter}<{n.iter.id}.len() {{")
                self.indentation+=1
                self.emit(f"let {temporary}={n.iter.id}.get({counter});")
                self.emit(f"assert!({temporary}.is_some()); // original complete ordered list traversal")
                self.emit(f"let {n.target.id}={temporary}.unwrap_or(Vec3::ZERO);")
                for statement in n.body:self.statement(statement)
                self.emit(f"{counter}={counter}.saturating_add(1);")
                self.indentation-=1;self.emit("}");return
            if not isinstance(n.iter,ast.Call) or ast.unparse(n.iter.func)!="range" or len(n.iter.args)!=1 or n.iter.keywords:
                raise ValueError("Unreviewed original loop")
            count=ast.literal_eval(n.iter.args[0])
            if type(count)is not int or not 1<=count<=2000:
                raise ValueError("Unreviewed original loop count")
            self.maximum_iterations=max(self.maximum_iterations,count)
            self.env[n.target.id]="int"
            counter=f"loop_iteration_{len(self.lines)}"
            self.emit(f"let mut {counter}=0_i64;")
            self.emit(f"let mut {n.target.id}=0_i64;")
            self.emit(f"while {counter} < {count}_i64 {{")
            self.indentation+=1
            self.emit(f"{n.target.id}={counter};")
            for statement in n.body: self.statement(statement)
            self.emit(f"{counter}={counter}.saturating_add(1);")
            self.indentation-=1
            self.emit("}")
            return
        if isinstance(n, ast.Expr) and isinstance(n.value, ast.Call):
            call = n.value
            if ast.unparse(call.func)=="helpers.add_ball_to_park":
                if len(call.args)!=1 or ast.unparse(call.args[0])!="self.park":raise ValueError("Unreviewed ignored constructor target")
                keywords={k.arg:k.value for k in call.keywords}
                if len(keywords)!=len(call.keywords):raise ValueError("Repeated fixture argument")
                self.add_ball("ignored_constructor_result_"+str(len(self.balls)),keywords)
                return
            if isinstance(call.func, ast.Attribute) and ast.unparse(call.func.value) == "self" and call.func.attr.startswith("assert"):
                name = call.func.attr
                self.emit(f"// original-assertion: {self.source}:{call.lineno}")
                if name == "assertListOfPointsAlmostEqual" and len(call.args)==2:
                    left,lk=self.expression(call.args[0]);right,rk=self.expression(call.args[1])
                    if not lk.startswith("points:") or not rk.startswith("points:"):
                        raise ValueError("Point comparison must preserve complete ordered lists")
                    places=4
                    for keyword in call.keywords:
                        if keyword.arg!="places": raise ValueError("Unreviewed point comparison keyword")
                        places=ast.literal_eval(keyword.value)
                    if type(places) is not int or not 0<=places<=6:
                        raise ValueError("Point rounding place count requires a separate numeric partition")
                    self.emit(f"assert_points_rounded(&{left}, &{right}, {places});")
                elif name in {"assertTrue", "assertFalse"} and len(call.args) in (1,2) and not call.keywords:
                    if len(call.args)==2:
                        message=call.args[1]
                        if not isinstance(message,ast.BinOp) or not isinstance(message.op,ast.Mod) or not isinstance(message.left,ast.Constant) or not isinstance(message.left.value,str) or not isinstance(message.right,ast.Tuple) or message.left.value.count('%f')!=len(message.right.elts):
                            raise ValueError("Unreviewed assertion diagnostic evaluation")
                        self.emit("let _diagnostic_arguments=("+", ".join(self.typed(v,"float") for v in message.right.elts)+"); // Python %f formatting/allocation remains a diagnostic boundary")
                    code, kind = self.expression(call.args[0])
                    if kind in {"int", "i8", "i32"}:
                        code = f"{code} != 0"
                    elif kind != "bool":
                        raise ValueError("Unreviewed assertion truth conversion")
                    self.emit("assert!(" + ("" if name == "assertTrue" else "!") + "(" + code + "));")
                elif name == "assertIsNone" and len(call.args) == 1:
                    code, kind = self.expression(call.args[0])
                    if kind not in {"park-association","optional-id","optional-boxes"}:
                        raise ValueError("Unreviewed None assertion type")
                    self.emit(f"assert!(({code}).is_none());")
                elif name in {"assertEqual", "assertNotEqual", "assertListEqual", "assertSetEqual", "assertDictEqual", "assertAlmostEqual", "assertIn", "assertNotIn"} and len(call.args) == 2:
                    left, lk = self.expression(call.args[0]); right, rk = self.expression(call.args[1])
                    if name in {"assertIn", "assertNotIn"}:
                        if rk not in {"ids","ball-map"}: raise ValueError("Unreviewed membership type")
                        predicate=f"{right}.contains({convert(left,lk,'int')})" if rk=="ball-map" else f"ids_contain(&{right}, {convert(left, lk, 'int')})"
                        self.emit("assert!(" + ("!" if name == "assertNotIn" else "") + predicate+");")
                    elif name == "assertAlmostEqual":
                        places = 7
                        delta=None
                        for k in call.keywords:
                            if k.arg=="places":places=ast.literal_eval(k.value)
                            elif k.arg=="delta":delta=self.typed(k.value,"float")
                            else:raise ValueError("Unreviewed numeric assertion keyword")
                        if delta is None:self.emit(f"assert!(python_almost_equal({convert(left, lk, 'float')}, {convert(right, rk, 'float')}, {places}) == Some(true));")
                        else:self.emit(f"assert!(destiny_original_spec::python_almost_equal_delta({convert(left,lk,'float')},{convert(right,rk,'float')},{delta}));")
                    else:
                        if call.keywords: raise ValueError("Unreviewed assertion keyword")
                        if lk != rk:
                            if lk=="optional-object" and rk=="object-dictionary":
                                right="Some("+right+")";rk="optional-object"
                            elif lk=="optional-object" and isinstance(call.args[1],ast.Constant) and call.args[1].value is None:
                                right="None::<OriginalBoxObject>";rk="optional-object"
                            elif lk=="optional-id" and rk in {"int","i8","i32","float"}:
                                right="Some("+convert(right,rk,"int")+")";rk="optional-id"
                            elif lk=="optional-id" and isinstance(call.args[1],ast.Constant) and call.args[1].value is None:
                                right="None::<i64>";rk="optional-id"
                            elif lk.startswith("points:") and rk.startswith("points:"):
                                pass # capacities differ; exact ordered comparison follows below
                            elif lk.startswith("point-set:") and rk.startswith("point-set:"):
                                pass
                            elif lk=="proximity-events" and isinstance(call.args[1],ast.List):
                                entries=[]
                                for item in call.args[1].elts:
                                    if not isinstance(item,ast.Tuple) or len(item.elts)!=2:raise ValueError("Original ordered proximity event tuple required")
                                    entries.append("("+", ".join(self.typed(v,"int") for v in item.elts)+")")
                                right="proximity_events(&["+", ".join(entries)+"])";rk="proximity-events"
                            elif lk=="distances" and rk=="ids" and isinstance(call.args[1],ast.List) and not call.args[1].elts:
                                right="distance_list(&[])";rk="distances"
                            elif rk=="distances" and lk=="ids" and isinstance(call.args[0],ast.List) and not call.args[0].elts:
                                left="distance_list(&[])";lk="distances"
                            elif lk in {"float", "int"} and rk in {"float", "int"}:
                                left, right = convert(left, lk, "float"), convert(right, rk, "float")
                            else: raise ValueError("Unreviewed equality types")
                        if name == "assertListEqual" and lk not in {"ids","distances"} and not lk.startswith("points:"): raise ValueError("List assertion must remain a list")
                        if lk.startswith("point-set:") and rk.startswith("point-set:"):
                            if name not in {"assertEqual","assertSetEqual"}:raise ValueError("Unreviewed point set assertion")
                            self.emit(f"assert!(point_sets_equal(&{left},&{right}));")
                        elif lk.startswith("points:") and rk.startswith("points:"):
                            if name not in {"assertEqual","assertListEqual"}:raise ValueError("Unreviewed point list comparison")
                            self.emit(f"assert!(points_exact_equal(&{left},&{right}));")
                        elif lk=="delta-map":
                            if name!="assertEqual":raise ValueError("Unreviewed dictionary comparison")
                            self.emit(f"assert!(delta_maps_equal(&{left},&{right}));")
                        elif lk=="id-set":
                            if name!="assertEqual": raise ValueError("Unreviewed set assertion")
                            self.emit(f"assert!(id_sets_equal(&{left}, &{right}));")
                        else: self.emit(f"assert!({left} {'!=' if name == 'assertNotEqual' else '=='} {right});")
                elif name in {"assertGreater","assertGreaterEqual","assertLess","assertLessEqual"} and len(call.args)==2 and not call.keywords:
                    operator={"assertGreater":">","assertGreaterEqual":">=","assertLess":"<","assertLessEqual":"<="}[name]
                    self.emit(f"assert!({self.typed(call.args[0],'float')} {operator} {self.typed(call.args[1],'float')});")
                elif name=="assertRaises" and len(call.args)>=2 and ast.unparse(call.args[0]) in {"RuntimeError","TypeError"}:
                    operation=ast.Call(func=call.args[1],args=call.args[2:],keywords=call.keywords)
                    self.expected_exception=True
                    try: code,kind=self.expression(operation)
                    finally: self.expected_exception=False
                    if kind not in {"result-distances","result-ids","result-updates"}: raise ValueError("Unreviewed runtime exception")
                    self.emit(f"assert!({code} == Err(OriginalParkError::{ast.unparse(call.args[0])}));")
                else:
                    raise ValueError("Unreviewed original assertion: " + name)
            else:
                if ast.unparse(call.func)=="self.park.SetBoxObject":
                    code,kind=self.expression(call)
                    if kind!="bool":raise ValueError("Original object setter result type")
                    self.emit("let _original_ignored_success="+code+"; // ignored original return; no-unexpected-exception check retained")
                    return
                if ast.unparse(call.func)=="self.check_transitions" and len(call.args)==1 and not call.keywords:
                    code,kind=self.expression(call.args[0])
                    if kind!="transitions":raise ValueError("Original transition sequence required")
                    self.emit("// original-assertion: python/destiny/test/helpers.py:288")
                    self.emit(f"assert!({code}==park.bubble_transitions); // complete original helper assertion")
                    return
                if isinstance(call.func,ast.Attribute) and call.func.attr=="append" and isinstance(call.func.value,ast.Name) and call.func.value.id in self.point_variables:
                    target=call.func.value.id
                    if len(call.args)!=1 or call.keywords or not isinstance(call.args[0],ast.Tuple) or len(call.args[0].elts)!=3:
                        raise ValueError("Unreviewed point append")
                    self.emit(f"assert!({target}.push({self.vec(call.args[0].elts)})); // every original trace element retained")
                    return
                code, kind = self.expression(call)
                if kind == "park-result":
                    self.emit(f"assert!({code} == Ok(())); // original call must not raise")
                elif kind == "unit":
                    self.emit(code + ";")
                else:
                    raise ValueError("Ignored call result")
            return
        if isinstance(n, ast.With) and len(n.items) == 1 and len(n.body) == 1:
            ctx = n.items[0].context_expr
            if not (isinstance(ctx, ast.Call) and ast.unparse(ctx.func) == "self.assertRaises" and len(ctx.args) == 1 and ast.unparse(ctx.args[0]) == "ValueError" and not ctx.keywords and n.items[0].optional_vars is None):
                raise ValueError("Unreviewed expected exception context")
            if not isinstance(n.body[0], ast.Expr) or not isinstance(n.body[0].value, ast.Call):
                raise ValueError("Expected exception body must retain one reviewed call")
            code, kind = self.expression(n.body[0].value)
            if kind != "park-result":
                raise ValueError("Unreviewed expected exception call")
            self.emit(f"// original-assertion: {self.source}:{ctx.lineno}")
            self.emit(f"assert!({code} == Err(OriginalParkError::ValueError));")
            return
        raise ValueError("Unreviewed statement: " + ast.unparse(n))


PREAMBLE = """// Copyright © 2015 CCP ehf. MIT license, retained from original Destiny tests.
// Generated from pinned Git blobs by generate_park_harnesses.py.
// Concrete state projection: 8 retained objects, 4096 ordered boundary calls.
// Spatial indexing, Blue dispatch/lifetime, interpreter and native libm
// correspondence remain required; no production refinement is claimed here.
#![allow(non_snake_case, unused_variables)]
use destiny_original_spec::{BoundedList, OriginalBallInit, OriginalBallAttribute, OriginalMode, OriginalParkBall, OriginalParkError, OriginalMiniBall, OriginalMiniCapsule, OriginalMiniBox,
    OriginalParkProjection, OriginalBubbleDelta, OriginalBubbleDeltaMap, OriginalBoxObject,OriginalBoxObjectKey,ActionValue,Vec3, python_almost_equal, python_rounded_equal};

fn box_object(values:&[(OriginalBoxObjectKey,ActionValue)]) -> OriginalBoxObject {
    let mut result=BoundedList::new();let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}

fn user_list(values:&[Option<i64>]) -> BoundedList<Option<i64>,8> {
    let mut result=BoundedList::new(); let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}
fn proximity_events(values:&[(i64,i64)]) -> BoundedList<(i64,i64),8> {
    let mut result=BoundedList::new(); let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}
fn transition_list(values:&[(i64,i64,i64)]) -> BoundedList<(i64,i64,i64),8> {
    let mut result=BoundedList::new(); let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}
fn delta_map(values:&[(i64,OriginalBubbleDelta)]) -> OriginalBubbleDeltaMap {
    let mut result=BoundedList::new(); let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}
fn delta_maps_equal(left:&OriginalBubbleDeltaMap,right:&OriginalBubbleDeltaMap) -> bool {
    if left.len()!=right.len() { return false; }
    let mut i=0;
    while i<left.len() {
        let value=left.get(i); let mut matches=0; let mut j=0;
        while j<right.len() { if value==right.get(j) { matches+=1; } j+=1; }
        if matches!=1 { return false; } i+=1;
    }
    true
}
fn store_delta_map(destination:&mut OriginalBubbleDeltaMap,value:OriginalBubbleDeltaMap) {
    *destination=value;
}

fn point_list<const N: usize>(points: &[Vec3]) -> BoundedList<Vec3,N> {
    let mut result=BoundedList::new();
    let mut i=0_usize;
    while i<points.len() {
        if let Some(point)=points.get(i) { assert!(result.push(*point)); }
        i=i.saturating_add(1);
    }
    result
}
fn points_exact_equal<const A:usize,const B:usize>(left:&BoundedList<Vec3,A>,right:&BoundedList<Vec3,B>) -> bool {
    if left.len()!=right.len() { return false; }
    let mut i=0;
    while i<left.len() { if left.get(i)!=right.get(i) { return false; } i+=1; }
    true
}
fn point_sets_equal<const A:usize,const B:usize>(left:&BoundedList<Vec3,A>,right:&BoundedList<Vec3,B>) -> bool {
    let mut i=0;
    while i<left.len() { let mut found=false;let mut j=0;
        while j<right.len() { found|=left.get(i)==right.get(j);j+=1; }if !found { return false; }i+=1;
    }
    i=0;
    while i<right.len() { let mut found=false;let mut j=0;
        while j<left.len() { found|=right.get(i)==left.get(j);j+=1; }if !found { return false; }i+=1;
    }
    true
}
fn sort_points<const N:usize>(values:&BoundedList<Vec3,N>) -> BoundedList<Vec3,N> {
    let mut result=*values;let mut i=1;
    while i<result.len() {
        let point=result.get(i);assert!(point.is_some());
        if let Some(point)=point { let mut j=i;
            while j>0 { let previous=result.get(j-1);assert!(previous.is_some());
                if let Some(previous)=previous {
                    if (previous.x,previous.y,previous.z)<=(point.x,point.y,point.z) { break; }
                    assert!(result.set(j,previous));
                }j-=1;
            }assert!(result.set(j,point));
        }i+=1;
    }result
}
// helpers.py:129..150 compares each rounded coordinate only for unequal
// points, then checks list lengths. Preserve that order and all three axes.
fn assert_points_rounded<const A:usize,const B:usize>(first:&BoundedList<Vec3,A>,second:&BoundedList<Vec3,B>,places:u32) {
    let limit=first.len().min(second.len());
    let mut i=0_usize;
    while i<limit {
        let left=first.get(i); let right=second.get(i);
        assert!(left.is_some() && right.is_some());
        if let (Some(left),Some(right))=(left,right) {
            if left!=right {
                assert!(python_rounded_equal(left.x,right.x,places)==Some(true));
                assert!(python_rounded_equal(left.y,right.y,places)==Some(true));
                assert!(python_rounded_equal(left.z,right.z,places)==Some(true));
            }
        }
        i=i.saturating_add(1);
    }
    assert!(first.len()==second.len());
}

fn id_list(values: &[i64]) -> BoundedList<i64, 8> {
    let mut result = BoundedList::new();
    let mut i = 0_usize;
    while i < values.len() {
        if let Some(value) = values.get(i) { assert!(result.push(*value)); }
        i = i.saturating_add(1);
    }
    result
}
fn ids_contain(values: &BoundedList<i64, 8>, id: i64) -> bool {
    let mut i = 0_usize;
    while i < values.len() {
        if values.get(i) == Some(id) { return true; }
        i = i.saturating_add(1);
    }
    false
}
fn distance_list(values:&[(f64,i64)]) -> BoundedList<(f64,i64),8> {
    let mut result=BoundedList::new(); let mut i=0_usize;
    while i<values.len() {
        if let Some(value)=values.get(i) { assert!(result.push(*value)); }
        i=i.saturating_add(1);
    }
    result
}
fn query_ids(values:&BoundedList<(f64,i64),8>) -> BoundedList<i64,8> {
    let mut result=BoundedList::new(); let mut i=0_usize;
    while i<values.len() {
        if let Some((_,id))=values.get(i) { assert!(result.push(id)); }
        i=i.saturating_add(1);
    }
    result
}
fn id_sets_equal(left:&BoundedList<i64,8>,right:&BoundedList<i64,8>) -> bool {
    let mut i=0_usize;
    while i<left.len() {
        if let Some(id)=left.get(i) { if !ids_contain(right,id) { return false; } }
        i=i.saturating_add(1);
    }
    i=0;
    while i<right.len() {
        if let Some(id)=right.get(i) { if !ids_contain(left,id) { return false; } }
        i=i.saturating_add(1);
    }
    true
}
"""


def translate(case):
    source, fixture, test = case.split("::")
    if source not in SOURCES:
        raise ValueError("Unreviewed source family")
    text = original(source)
    tree = ast.parse(text)
    cls = next(c for c in tree.body if isinstance(c, ast.ClassDef) and c.name == fixture)
    base=["helpers.BallparkWrappedTestCase"] if source.endswith("test_lifecycle_management.py") and fixture=="TestBubbleTransitions" else ["helpers.BallparkTestCase"]
    if cls.bases and [ast.unparse(b) for b in cls.bases] != base:
        raise ValueError("Unreviewed fixture chain")
    if base==["helpers.BallparkWrappedTestCase"]:
        helper_tree=ast.parse(original("python/destiny/test/helpers.py"))
        wrapped=next(n for n in helper_tree.body if isinstance(n,ast.ClassDef) and n.name=="BallparkWrappedTestCase")
        actual=next(n for n in wrapped.body if isinstance(n,ast.FunctionDef) and n.name=="check_transitions")
        expected=ast.parse("def check_transitions(self,expected_transitions):\n actual_transitions=self.park.GetTransitions()\n self.assertEqual(expected_transitions,actual_transitions)\n").body[0]
        if ast.dump(actual)!=ast.dump(expected):raise ValueError("Reviewed transition helper changed")
    overrides = [n for n in cls.body if isinstance(n, ast.FunctionDef) and n.name in {"setUp", "tearDown"}]
    id_fixture = source.endswith("test_lifecycle_management.py") and fixture == "TestIdGeneration"
    goto_fixture = source.endswith("evolve/test_goto.py") and fixture == "TestGoto"
    simple_fixture=source.endswith("evolve/test_simple_collision.py") and fixture=="TestSimpleCollision"
    static_collision_fixture=source.endswith("evolve/test_simple_collision.py") and fixture in {"TestSimpleCapsuleCollision","TestSimpleBoxCollision"}
    formation_fixture=source.endswith("evolve/test_follow.py") and fixture=="TestFormation"
    warp_fixture=source.endswith("evolve/test_warp.py") and fixture=="TestWarp"
    missile_fixture=source.endswith("evolve/test_missile.py") and fixture=="TestMissile"
    callback_fixture=source.endswith("test_callbacks.py") and fixture=="TestSetBallNotInParkCallback"
    if overrides and not (id_fixture or goto_fixture or simple_fixture or callback_fixture or static_collision_fixture or formation_fixture or warp_fixture or missile_fixture):
        raise ValueError("Unreviewed fixture override")
    body = next(n for n in cls.body if isinstance(n, ast.FunctionDef) and n.name == test)
    if body.decorator_list:
        raise ValueError("Unreviewed original decorators")
    t = Translation(source)
    if missile_fixture:
        expected_setup=ast.parse("def setUp(self):\n super(TestMissile,self).setUp()\n self.dst=helpers.create_space_ball(self.park,x=1e6)\n self.owner=helpers.create_space_ball(self.park)\n self.collider=helpers.create_space_ball(self.park,x=2e3)\n self.collider.radius=1.9e3\n self.missile=helpers.create_space_ball(self.park)\n self.missile.maxVelocity=300\n self.missile.Agility=0.01\n try:\n  self.missile.maxAngularVelocity=2.0\n  self.missile.RotationalAgility=0.5\n except AttributeError:\n  pass\n self.missile.mass=1e4\n").body[0]
        expected_teardown=ast.parse("def tearDown(self):\n super(TestMissile,self).tearDown()\n del self.dst\n del self.owner\n del self.collider\n del self.missile\n").body[0]
        if len(overrides)!=2 or [ast.dump(x) for x in overrides]!=[ast.dump(expected_setup),ast.dump(expected_teardown)]:raise ValueError("Reviewed missile lifecycle changed")
        class BindFixture(ast.NodeTransformer):
            def visit_Attribute(self,node):
                if isinstance(node.value,ast.Name) and node.value.id=="self" and node.attr in {"dst","owner","collider","missile"}:
                    return ast.copy_location(ast.Name(id="fixture_"+node.attr,ctx=node.ctx),node)
                return self.generic_visit(node)
        body=BindFixture().visit(copy.deepcopy(body))
        t.evolve_method="evolve_two_spherical_bodies";t.maximum_iterations=128
        t.env["next_space_object_id"]="int";t.emit("let mut next_space_object_id=1_i64;")
        for statement in overrides[0].body[1:]:t.statement(BindFixture().visit(copy.deepcopy(statement)))
    if warp_fixture:
        expected_setup=ast.parse("def setUp(self):\n self.maxDiff=None\n super(TestWarp,self).setUp()\n").body[0]
        if len(overrides)!=1 or ast.dump(overrides[0])!=ast.dump(expected_setup):raise ValueError("Reviewed Warp fixture changed")
        t.evolve_method="evolve_two_spherical_bodies";t.maximum_iterations=128
    if source.endswith("test_visibility.py") and test=="test_uncloaking_a_ball_in_warp_does_not_make_it_massive":
        t.evolve_method="evolve_two_spherical_bodies";t.maximum_iterations=128
    if source.endswith("evolve/test_minis.py") and test in {"test_collide","test_collide_with_miniballs"}:
        t.evolve_method="evolve_two_spherical_bodies";t.maximum_iterations=128
        helper_tree=ast.parse(original("python/destiny/test/helpers.py"));base=next(n for n in helper_tree.body if isinstance(n,ast.ClassDef) and n.name=="BallparkTestCase")
        helper=next(n for n in base.body if isinstance(n,ast.FunctionDef) and n.name=="evolve_ball_and_get_coordinates")
        expected=ast.parse("def evolve_ball_and_get_coordinates(self,ball,step_count):\n coordinates=[]\n for i in range(step_count):\n  self.park.Evolve()\n  coordinates.append((ball.x,ball.y,ball.z))\n return coordinates\n").body[0]
        if ast.dump(helper)!=ast.dump(expected):raise ValueError("Reviewed complete evolution helper changed")
        t.helpers["self.evolve_ball_and_get_coordinates"]=helper;t.point_variables.add("coordinates")
        counts=[ast.literal_eval(n.args[1]) for n in ast.walk(body) if isinstance(n,ast.Call) and ast.unparse(n.func)=="self.evolve_ball_and_get_coordinates" and len(n.args)==2]
        if counts!=[20]:raise ValueError("Reviewed mini collision helper counts changed")
        t.point_capacity=20
    if formation_fixture:
        expected_setup=ast.parse("def setUp(self):\n super(TestFormation,self).setUp()\n self.park.LoadFormations(helpers.FORMATIONS)\n self.maxDiff=None\n").body[0]
        expected_teardown=ast.parse("def tearDown(self):\n self.park.ClearAll()\n del self.park\n").body[0]
        if len(overrides)!=2 or [ast.dump(x) for x in overrides]!=[ast.dump(expected_setup),ast.dump(expected_teardown)]:
            raise ValueError("Reviewed formation evolution lifecycle changed")
        helper_tree=ast.parse(original("python/destiny/test/helpers.py"))
        formations=next(x.value for x in helper_tree.body if isinstance(x,ast.Assign) and any(isinstance(y,ast.Name) and y.id=="FORMATIONS" for y in x.targets))
        if ast.literal_eval(formations)!=(('Diamond',((100.0,0.0,0.0),(0.0,100.0,0.0),(-100.0,0.0,0.0),(0.0,-100.0,0.0))),('Arrow',((100.0,0.0,-50.0),(50.0,0.0,0.0),(-100.0,0.0,-50.0),(-50.0,0.0,0.0))),('FormationOfOne',((0.0,0.0,0.0),))):
            raise ValueError("Reviewed formation point table changed")
        t.emit("park.load_original_formations();")
        t.evolve_method="evolve_two_spherical_bodies";t.maximum_iterations=128
    if source.endswith("test_boxes_and_bubbles.py"):
        t.point_capacity=max([1]+[len(n.elts) for n in ast.walk(body) if isinstance(n,ast.List) and n.elts and all(isinstance(x,ast.Tuple) and len(x.elts)==3 for x in n.elts)])
        t.allow_canonical_boxes=any(isinstance(n,ast.Call) and ast.unparse(n.func) in {"set","sorted"} for n in ast.walk(body))
    if callback_fixture:
        expected_setup=ast.parse("def setUp(self):\n self.called=False\n super(TestSetBallNotInParkCallback,self).setUp()\n").body[0]
        handler=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=="handle_ball_not_in_park_error")
        expected_handler=ast.parse("def handle_ball_not_in_park_error(self,ball):\n self.called=True\n").body[0]
        if len(overrides)!=1 or ast.dump(overrides[0])!=ast.dump(expected_setup) or ast.dump(handler)!=ast.dump(expected_handler):
            raise ValueError("Reviewed callback lifecycle/handler changed")
        t.mutable_detached={ast.unparse(n.func.value) for n in ast.walk(body) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute) and n.func.attr=="AddMiniBall"}
        t.emit("park.ball_not_in_park_callback_called=false; // original self.called fixture initialization")
    if source.endswith("test_callbacks.py") and not callback_fixture:
        helper_tree=ast.parse(original("python/destiny/test/helpers.py"))
        spy=next(n for n in helper_tree.body if isinstance(n,ast.ClassDef) and n.name=="BallEventSpy")
        expected=ast.parse("class BallEventSpy(destiny.Ball):\n def __init__(self,*args):\n  super(BallEventSpy,self).__init__(*args)\n  self.collision_callback_args=[]\n  self.proximity_callback_args=[]\n  self.range_callback_args=[]\n  self.target_tracking_callback_args=[]\n def DoProximity(self,intruderID,entered):\n  self.proximity_callback_args.append((intruderID,entered))\n def DoRange(self,in_range):\n  self.range_callback_args.append(in_range)\n def DoTargetTracking(self,entered):\n  self.target_tracking_callback_args.append(entered)\n def DoCollision(self,id,x,y,z):\n  self.collision_callback_args.append((id,x,y,z))\n").body[0]
        if ast.dump(spy)!=ast.dump(expected):raise ValueError("Reviewed append-only event spy changed")
        t.evolve_method="evolve_two_spherical_bodies"
    if simple_fixture:t.evolve_method="evolve_two_spherical_bodies"
    if static_collision_fixture:
        if fixture=="TestSimpleCapsuleCollision":
            expected_setup=ast.parse("def setUp(self):\n super(TestSimpleCapsuleCollision,self).setUp()\n parentBall=helpers.add_ball_to_park(self.park,x=0.0,y=0.0,z=0.0,radius=500.0)\n self.capsule=parentBall.AddMiniCapsule(100.0,100.0,0.0,200.0,100.0,0.0,1.0)\n").body[0]
            t.fixture_native_none={"self.capsule"}
        else:
            expected_setup=ast.parse("def setUp(self):\n super(TestSimpleBoxCollision,self).setUp()\n parentBall=helpers.add_ball_to_park(self.park,x=0,y=0,z=0,radius=500.0)\n self.box=parentBall.AddMiniBox(0.0,0.0,0.0,100.0,0.0,0.0,0.0,100.0,0.0,0.0,0.0,100.0)\n").body[0]
            t.fixture_native_none={"self.box"}
        if len(overrides)!=1 or ast.dump(overrides[0])!=ast.dump(expected_setup):raise ValueError("Reviewed static collision fixture changed")
        t.evolve_method="evolve_two_spherical_bodies";t.maximum_iterations=128
        helper_tree=ast.parse(original("python/destiny/test/helpers.py"));base=next(n for n in helper_tree.body if isinstance(n,ast.ClassDef) and n.name=="BallparkTestCase")
        helper=next(n for n in base.body if isinstance(n,ast.FunctionDef) and n.name=="evolve_ball_and_get_coordinates")
        expected_helper=ast.parse("def evolve_ball_and_get_coordinates(self,ball,step_count):\n coordinates=[]\n for i in range(step_count):\n  self.park.Evolve()\n  coordinates.append((ball.x,ball.y,ball.z))\n return coordinates\n").body[0]
        if ast.dump(helper)!=ast.dump(expected_helper):raise ValueError("Reviewed complete evolution helper changed")
        t.helpers["self.evolve_ball_and_get_coordinates"]=helper
        t.point_variables.add("coordinates")
        t.point_capacity=max([1]+[len(n.elts) for n in ast.walk(body) if isinstance(n,ast.List)])
        helper_counts=[ast.literal_eval(n.args[1]) for n in ast.walk(body) if isinstance(n,ast.Call) and ast.unparse(n.func)=="self.evolve_ball_and_get_coordinates" and len(n.args)==2]
        if not helper_counts or any(type(count)is not int or not 1<=count<=2000 for count in helper_counts):raise ValueError("Unreviewed helper iteration count")
        t.point_capacity=max([t.point_capacity]+helper_counts)
        for statement in overrides[0].body[1:]:t.statement(statement)
    if source.endswith("evolve/test_follow.py") and fixture=="TestFollow":t.evolve_method="evolve_two_spherical_bodies"
    if source.endswith("evolve/test_orbit.py") and fixture=="TestOldOrbit":t.evolve_method="evolve_two_spherical_bodies"
    if source.endswith("test_boxes_and_bubbles.py") and fixture=="TestGetBallIdsInCone":
        helper=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=="_get_colliding_balls")
        t.helpers={"self._get_colliding_balls":helper}
    if source.endswith("test_lifecycle_management.py") and fixture=="TestAddMushroom":
        t.evolve_method="evolve_owner_and_mushroom"
    point_appends=[n for n in ast.walk(body) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute) and n.func.attr=="append" and isinstance(n.func.value,ast.Name) and len(n.args)==1 and isinstance(n.args[0],ast.Tuple) and len(n.args[0].elts)==3]
    t.point_variables.update(n.func.value.id for n in point_appends)
    if point_appends:
        lengths=[len(n.elts) for n in ast.walk(body) if isinstance(n,ast.List)]
        loop_counts=[ast.literal_eval(n.iter.args[0]) for n in ast.walk(body) if isinstance(n,ast.For) and isinstance(n.iter,ast.Call) and ast.unparse(n.iter.func)=="range" and len(n.iter.args)==1]
        t.point_capacity=max([1]+lengths+loop_counts)
    if goto_fixture or simple_fixture:
        expected_setup=ast.parse(f"def setUp(self):\n self.maxDiff=None\n super({fixture},self).setUp()\n").body[0]
        if len(overrides)!=1 or ast.dump(overrides[0])!=ast.dump(expected_setup):
            raise ValueError("Reviewed Goto fixture changed")
    if any(isinstance(n,ast.Call) and ast.unparse(n.func)=="helpers.create_space_ball" for n in ast.walk(body)):
        t.env["next_space_object_id"]="int"
        t.emit("// Fresh Python helper module starts its next_object_id counter at 1.")
        t.emit("let mut next_space_object_id=1_i64;")
    if id_fixture:
        expected_setup = ast.parse("def setUp(self):\n super(TestIdGeneration, self).setUp()\n self.first_id_generated = -2**30 - 1\n").body[0]
        if len(overrides) != 1 or ast.dump(overrides[0], include_attributes=False) != ast.dump(expected_setup, include_attributes=False):
            raise ValueError("Reviewed ID fixture changed")
        t.env["first_id_generated"] = "int"
        t.emit("let first_id_generated = -(1_i64 << 30) - 1;")
        helpers = {n.name:n for n in cls.body if isinstance(n, ast.FunctionDef) and n.name in {"get_next_ball_id", "get_next_capsule_id", "get_next_oriented_box_id"}}
        if set(helpers) != {"get_next_ball_id", "get_next_capsule_id", "get_next_oriented_box_id"}:
            raise ValueError("ID fixture helper inventory changed")
        t.helpers = {"self." + name:fn for name,fn in helpers.items()}
    for n in body.body:
        t.statement(n)
    function = Path(source).stem.removeprefix("test_") + "__" + fixture + "__" + test
    proof = f"\n// original-case: {case}\n// original-source-sha256: {hashlib.sha256(text.encode()).hexdigest()}\n"
    unwind=max(17,t.maximum_iterations+1)
    proof += f"#[cfg_attr(kani, kani::proof)]\n#[cfg_attr(kani, kani::unwind({unwind}))]\n#[cfg_attr(test, test)]\n"
    proof += f"fn {function}() {{\n    let mut park = OriginalParkProjection::default();\n"
    proof += "\n".join(t.lines) + "\n"
    proof += "    assert!(!park.scope_error);\n    park.clear_all();\n    assert!(!park.scope_error);\n    #[cfg(kani)]\n    kani::cover!(true, \"original park projection scenario and teardown completed\");\n}\n"
    return proof


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--refresh",action="store_true",help="Regenerate only the corresponding existing OPEN case from pinned AST")
    args = parser.parse_args()
    if args.output is None:
        args.output=ROOT/"verification/kani/src/park_evolution_contracts.rs" if "/ballpark/evolve/" in args.case else TARGET
    import json
    focus=json.loads((ROOT/".aegis/destiny/harness-focus.json").read_text())
    if focus.get("status")!="OPEN" or focus.get("request",{}).get("case")!=args.case:
        raise ValueError("Corresponding OPEN Aegis work packet required")
    proof = translate(args.case)  # fail closed before touching output
    old = args.output.read_text() if args.output.exists() else PREAMBLE
    marker="// original-case: "+args.case+"\n"
    if marker in old:
        if not args.refresh:raise ValueError("Case already translated")
        start=old.index(marker)
        end=old.find("\n// original-case: ",start+len(marker))
        if end<0:end=len(old)
        old=old[:start]+proof.lstrip("\n")+old[end:]
        proof=""
    elif args.refresh:raise ValueError("Refresh requires an existing concrete case")
    # This file's shared prelude is generator-owned. Refresh it when a reviewed
    # helper is added; existing case bodies and qualified identities survive.
    first_case=old.find("\n// original-case: ")
    if first_case>=0:
        old=PREAMBLE+old[first_case:]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(old + proof)
    print("Translated", args.case)


if __name__ == "__main__":
    main()
