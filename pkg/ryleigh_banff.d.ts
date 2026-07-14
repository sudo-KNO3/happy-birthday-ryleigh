/* tslint:disable */
/* eslint-disable */

/**
 * Oil-paint an arbitrary RGBA8 image (e.g. a real Banff photo pulled from a
 * `<canvas>` via `getImageData`). Returns a new stylised buffer of the same
 * size; if the length doesn't match `width*height*4` the input is returned
 * unchanged so the page can fail gracefully.
 */
export function oil_paint(bytes: Uint8Array, width: number, height: number): Uint8Array;

/**
 * WebAssembly entry point. Returns an RGBA8 buffer the page wraps in an
 * `ImageData` and blits to a `<canvas>`.
 */
export function render(width: number, height: number, seed: number, kick: boolean): Uint8Array;

/**
 * Oil-paint a photo and, if `kick` is set, drop the birthday roundhouse cameo
 * into the foreground. Used by the site to turn the Banff photo into a
 * painterly keepsake with the inside joke on tap.
 */
export function stylize(bytes: Uint8Array, width: number, height: number, kick: boolean): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly oil_paint: (a: number, b: number, c: number, d: number) => [number, number];
    readonly render: (a: number, b: number, c: number, d: number) => [number, number];
    readonly stylize: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
