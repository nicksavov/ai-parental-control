/* tslint:disable */
/* eslint-disable */

export function buildBundle(identity_json: string, prekey_json: string): string;

export function generateIdentity(): string;

export function generatePrekey(): string;

export function initiatorHandshake(identity_json: string, bundle_json: string): string;

/**
 * Open an alert envelope back into a validated alert (JSON). This is what the
 * parent dashboard calls to read an alert in the browser.
 */
export function openAlert(shared_secret: string, envelope_json: string): string;

export function responderHandshake(identity_json: string, prekey_secret: string, initiator_dh_public: string, ephemeral_public: string): string;

export function sealAlert(shared_secret: string, alert_json: string, recipient_id: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly buildBundle: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly generateIdentity: () => [number, number, number, number];
    readonly generatePrekey: () => [number, number, number, number];
    readonly initiatorHandshake: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly openAlert: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly responderHandshake: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number, number];
    readonly sealAlert: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
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
