// Ambient types for Vite's `?worker` import suffix. Vite rewrites these imports
// to a constructable Worker class at build time; tsc needs the declaration.
declare module "*?worker" {
  const WorkerFactory: new () => Worker;
  export default WorkerFactory;
}
