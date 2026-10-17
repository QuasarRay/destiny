#!/usr/bin/env python3
"""Validate emitted XML with its own schemas and preserve unresolved source joins."""
import hashlib
import json
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
DIRECTORY = ROOT / ".aegis/destiny/migration/doxygen/xml"
ORIGINAL = ROOT / ".aegis/destiny/original-formation-ci"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inspect():
    inventory = json.loads((ROOT / "verification/migration/source_inventory.json").read_text())
    sources = {row["path"]: row for row in inventory["revisions"][-1]["files"] if row["kind"] == "blob"}
    for path, row in sources.items():
        if not (ORIGINAL / path).is_file() or sha(ORIGINAL / path) != row["sha256"]:
            raise ValueError("documentation source tree differs from authoritative inventory: " + path)
    schemas = {"doxygen": "compound.xsd", "doxygenindex": "index.xsd", "doxyfile": "doxyfile.xsd"}
    files, records, unresolved = [], [], []
    for path in sorted(DIRECTORY.glob("*.xml")):
        tree = ET.parse(path)
        root = tree.getroot()
        if root.tag not in schemas:
            raise ValueError("unknown XML root: " + root.tag)
        schema = DIRECTORY / schemas[root.tag]
        checked = subprocess.run(["xmllint", "--nonet", "--noout", "--schema", str(schema), str(path)], capture_output=True, text=True)
        if checked.returncode:
            raise ValueError("XML schema validation failed: " + checked.stderr[-1500:])
        files.append({"file": path.name, "sha256": sha(path), "schema": schema.name, "schema_sha256": sha(schema)})
        for compound in root.findall("compounddef"):
            name = compound.findtext("compoundname", "")
            for member in compound.findall(".//memberdef"):
                location = member.find("location")
                member_name = member.findtext("qualifiedname") or name + "::" + member.findtext("name", "")
                source = location.get("file", "") if location is not None else ""
                if source.startswith(str(ORIGINAL) + "/"):
                    source = str(Path(source).relative_to(ORIGINAL))
                elif source.startswith(str(ORIGINAL.relative_to(ROOT)) + "/"):
                    source = str(Path(source).relative_to(ORIGINAL.relative_to(ROOT)))
                signature = member.findtext("argsstring", "")
                line = location.get("line") if location is not None else None
                record = {"id": f"{inventory['authoritative_commit']}::{source}::{member_name}::{signature}::{line}",
                          "qualified_name": member_name, "signature": signature, "kind": member.get("kind"),
                          "access": member.get("prot"), "static": member.get("static"), "source": source,
                          "line": int(line) if line else None, "source_sha256": sources.get(source, {}).get("sha256"),
                          "documentation_graph_id": member.get("id"), "fact_kind": "doxygen_documentation",
                          "compiler_join": "UNRESOLVED; USR/signature/configuration join required",
                          "return_type": "".join(member.find("type").itertext()) if member.find("type") is not None else "",
                          "initializer": "".join(member.find("initializer").itertext()) if member.find("initializer") is not None else None}
                if source not in sources:
                    unresolved.append({"id": record["id"], "reason": "XML location has no authoritative original source identity"})
                records.append(record)
    indexed = {node.get("refid") + ".xml" for node in ET.parse(DIRECTORY / "index.xml").getroot().findall("compound")}
    actual = {row["file"] for row in files}
    if not indexed or not indexed <= actual:
        raise ValueError("missing indexed compound XML records")
    ids = [record["id"] for record in records]
    duplicates = sorted({identity for identity in ids if ids.count(identity) > 1})
    report = {"schema": 1, "original_commit": inventory["authoritative_commit"],
              "source_tree_sha256": inventory["revisions"][-1]["source_tree_sha256"],
              "doxyfile_sha256": sha(ROOT / "verification/migration/Doxyfile"), "generator_sha256": sha(Path(__file__)),
              "status": "SCHEMA_VALIDATED_DOCUMENTATION_ONLY", "xml_files": files,
              "indexed_compounds": len(indexed), "records": records,
              "unresolved_source_joins": unresolved, "ambiguous_identity_joins": duplicates,
              "semantic_extraction_complete": False,
              "notice": "All original file bytes checked; index and compound records validate against emitted XSD. Documentation records do not establish compiler-resolved or behavioral completeness."}
    (ROOT / "verification/migration/documentation_inventory.json").write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "xml_files": len(files), "indexed_compounds": len(indexed), "documentation_records": len(records), "unresolved_source_joins": len(unresolved), "ambiguous_identity_joins": len(duplicates)}))


if __name__ == "__main__":
    inspect()
