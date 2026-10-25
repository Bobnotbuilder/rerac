#!/usr/bin/env python3
"""Group actual native-world initializer owners and static original dependencies.

Inputs are private metadata from original_aranos's RC2_INITIALIZER_INVENTORY
and PS2Recomp's call relationships. No original payload is copied to source.
Static call-site order does not establish branch execution or function coverage.
"""
import argparse
from collections import defaultdict
import json
import re

# Evidence scope, not blanket callback support. Keep branches explicit.
NATIVE = {
    0x383b90: "COMPLETE_DELETE_CALLBACK",
    0x383bb0: "STATE0_LINK_WAIT_SUPPORTED_STATE2_EFFECTS_MISSING",
    0x383ce0: "STATE0_TYPE89_RESOURCE_DEPENDENT_LATER_DESTRUCTION_MISSING",
    0x383eb8: "STATE0_SUPPORTED_STATE1_QUERY_AND_DRAW_MISSING",
    0x3846d0: "STATE0_PREFIX_ONLY_TELEPORT_AND_TAIL_REQUIRED",
}

def address(value):
    if isinstance(value, int):
        return value
    match = re.fullmatch(r"(?:sub_)?(?:0x)?([0-9a-fA-F]{1,8})", value)
    if not match:
        raise ValueError("invalid source address")
    return int(match.group(1), 16)

def inventory(world, calls):
    rows = world["initializers"]
    groups = {}
    previous = -1
    for ordinal, row in enumerate(rows):
        if row["ordinal"] != ordinal or row["owner"] <= previous:
            raise ValueError("initializer owner order is not the loader sequence")
        previous = row["owner"]
        source = None if row["source"] is None else address(row["source"])
        group = groups.setdefault(source, {"source": source, "owners": [], "classes": set()})
        group["owners"].append(row["owner"])
        group["classes"].add(address(row["class"]))
    edges = defaultdict(list)
    for call in calls:
        try:
            callee = f"{address(call['callee']):08x}"
        except ValueError:
            if not re.fullmatch(r"[A-Za-z_][A-Za-z_0-9]*", call["callee"]):
                raise ValueError("invalid external symbol")
            callee = "symbol:" + call["callee"]
        edges[address(call["caller"])].append((address(call["site"]), callee))
    shared = defaultdict(set)
    result = []
    for source, group in groups.items():
        sites = sorted(set(edges.get(source, [])))
        for _, callee in sites:
            shared[callee].add(source)
        result.append({"source": None if source is None else f"{source:08x}",
                       "owners": group["owners"],
                       "classes": [f"{c:04x}" for c in sorted(group["classes"])],
                       "native_scope": NATIVE.get(source, "UNKNOWN_REQUIRES_EVIDENCE"),
                       "direct_static_calls": [{"site": f"{s:08x}", "callee": c} for s,c in sites]})
    shared_rows = [{"service": callee,
                    "callback_sources": [f"{s:08x}" for s in sorted(sources)]}
                   for callee, sources in sorted(shared.items()) if len(sources)>1]
    return {"status": "DEPENDENCY_BATCH_INVENTORY_NOT_CALLBACK_OR_GAMEPLAY_VALIDATION",
            "provenance": world.get("provenance", "UNSPECIFIED"),
            "owner_count": len(rows), "callback_groups": result,
            "shared_direct_services": shared_rows,
            "limitations": ["Original runtime inputs are explicit synthetic fixtures, not New Game",
                            "Grouping never bypasses class parameters, pvars, RNG or lifetime",
                            "Static call sites are not an execution trace; indirect calls and inferred boundaries remain incomplete",
                            "Null source does not establish that all initialization requirements are absent"]}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("world_metadata")
    parser.add_argument("original_call_relationships")
    parser.add_argument("private_report")
    args=parser.parse_args()
    with open(args.world_metadata) as f: world=json.load(f)
    with open(args.original_call_relationships) as f: calls=json.load(f)
    report=inventory(world,calls)
    with open(args.private_report,"w") as f: json.dump(report,f,indent=2);f.write("\n")
    print("INITIALIZER_DEPENDENCY_INVENTORY",report["owner_count"],len(report["callback_groups"]),len(report["shared_direct_services"]))

if __name__=="__main__": main()
