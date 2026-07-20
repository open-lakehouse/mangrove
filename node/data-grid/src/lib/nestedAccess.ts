// Zero-copy navigation into nested Arrow struct columns. arrow-js exposes struct
// children as child Vectors via `Vector.getChild(name)` — a view over the same
// backing buffers, no copy — and struct leaves are enumerable on the type via
// `type.children: Field[]`. These helpers keep that navigation in one place so
// the store and the visualizations never re-implement the walk (or accidentally
// materialize a whole struct with `.toJSON()`).

import type { DataType, Field, Vector } from "apache-arrow";

/**
 * Walk a struct path down from a root Vector, returning the leaf child Vector or
 * `null` if any segment is absent. Each hop is `Vector.getChild(name)`, which
 * returns a zero-copy view over the child's buffers. An absent segment (e.g. a
 * `minValues` struct that the writer never emitted) short-circuits to `null`, so
 * callers get one uniform "not present" signal.
 */
export function resolveChildPath(
  root: Vector | null,
  path: readonly string[],
): Vector | null {
  let vec: Vector | null = root;
  for (const name of path) {
    if (!vec) return null;
    // getChild returns null when the field name isn't a child of this struct
    // (or the vector isn't a struct at all).
    vec = (vec.getChild(name) as Vector | null) ?? null;
  }
  return vec;
}

/**
 * Look up a child field by name on a struct-typed DataType, or `null` if the
 * type has no such child (or isn't a struct). Schema-side companion to
 * {@link resolveChildPath} — use it to test presence and read a leaf's type
 * without touching any data.
 */
export function structFieldByName(
  type: DataType | null | undefined,
  name: string,
): Field | null {
  const children = (type as { children?: Field[] } | null | undefined)
    ?.children;
  if (!children) return null;
  return children.find((f) => f.name === name) ?? null;
}

/** The child fields of a struct-typed DataType, or `[]` if it isn't a struct. */
export function structChildren(
  type: DataType | null | undefined,
): readonly Field[] {
  return (type as { children?: Field[] } | null | undefined)?.children ?? [];
}
