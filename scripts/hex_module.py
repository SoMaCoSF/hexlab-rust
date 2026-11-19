#!/usr/bin/env python3
"""HexLab — procedural HEX module generator (Blender, headless).

This is why HexLab is a LOCAL app: generation runs on the machine that
has Blender. The Rust app (hexlab-core `generate::run_job`) invokes:

    blender --background --factory-startup --python hex_module.py -- \
        --module core --size standard --ff 32 --length 12.2 \
        --corridor-ff 8.96 --out /path/to/module.glb

Builds the hex-tube kit's base shell: an open hexagonal tube — outer
wall, inner corridor wall, annular end rings — flat-to-flat = --ff,
axis along Z then rotated so the tube lies horizontal (axis = Y in the
export, matching the web configurator's sea-plane). Module-specific
fit-out (flight gate, moonpool, dry-dock floor) layers on top of this
shell in the AnyMachine pipeline; the shell is the shared substrate
every HEX module in the library starts from.

No add-ons required. Exports .glb (default) or .usd by extension.
"""
import argparse
import math
import sys

import bpy


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    p = argparse.ArgumentParser()
    p.add_argument("--module", default="core")
    p.add_argument("--size", default="standard")
    p.add_argument("--ff", type=float, default=32.0, help="flat-to-flat metres")
    p.add_argument("--length", type=float, default=12.2)
    p.add_argument("--corridor-ff", type=float, default=0.0,
                   help="corridor flat-to-flat metres (0 = 0.28 * ff)")
    p.add_argument("--out", required=True)
    return p.parse_args(argv)


MODULE_COLORS = {
    "core": (0.09, 0.31, 0.39), "flight": (0.96, 0.62, 0.04),
    "drydock": (0.06, 0.72, 0.51), "subhub": (0.65, 0.33, 0.97),
    "refinery": (0.94, 0.27, 0.27), "desal": (0.23, 0.51, 0.96),
    "lab": (0.89, 0.91, 0.94), "power": (0.98, 0.44, 0.53),
}


def hex_ring(radius, z):
    """Six vertices of a pointy-top hexagon at height z."""
    return [
        (radius * math.cos(math.radians(60 * i - 30)),
         radius * math.sin(math.radians(60 * i - 30)), z)
        for i in range(6)
    ]


def build_shell(ff, length, corridor_ff):
    r_out = ff / math.sqrt(3.0)
    r_in = corridor_ff / math.sqrt(3.0)
    z0, z1 = -length / 2.0, length / 2.0
    outer0, outer1 = hex_ring(r_out, z0), hex_ring(r_out, z1)
    inner0, inner1 = hex_ring(r_in, z0), hex_ring(r_in, z1)
    verts = outer0 + outer1 + inner0 + inner1  # 24 verts
    faces = []
    for i in range(6):
        j = (i + 1) % 6
        faces.append((i, j, 6 + j, 6 + i))                 # outer wall
        faces.append((12 + i, 18 + i, 18 + j, 12 + j))     # inner wall
        faces.append((i, 12 + i, 12 + j, j))               # end ring z0
        faces.append((6 + i, 6 + j, 18 + j, 18 + i))       # end ring z1
    mesh = bpy.data.meshes.new("HexTube")
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    return mesh


def main():
    args = parse_args()
    corridor_ff = args.corridor_ff or args.ff * 0.28

    bpy.ops.wm.read_factory_settings(use_empty=True)
    mesh = build_shell(args.ff, args.length, corridor_ff)
    obj = bpy.data.objects.new(f"hex_{args.module}_{args.size}", mesh)
    bpy.context.collection.objects.link(obj)

    color = MODULE_COLORS.get(args.module, (0.5, 0.5, 0.5))
    mat = bpy.data.materials.new(name=f"hexlab_{args.module}")
    mat.diffuse_color = (*color, 1.0)
    mesh.materials.append(mat)

    # Lie the tube horizontal: axis Z -> Y.
    obj.rotation_euler = (math.radians(90), 0, 0)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=False)

    if args.out.lower().endswith(".usd"):
        bpy.ops.wm.usd_export(filepath=args.out)
    else:
        bpy.ops.export_scene.gltf(filepath=args.out, export_format="GLB")
    print(f"HEXLAB_OK module={args.module} size={args.size} ff={args.ff} out={args.out}")


if __name__ == "__main__":
    main()
