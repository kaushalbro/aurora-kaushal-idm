/* tslint:disable */
/* eslint-disable */

export class AuroraWasmEngine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Wall-clock end of the download (JS epoch millis), if completed.
     */
    completed_at_ms(): bigint | undefined;
    /**
     * Elapsed wall-clock seconds from Rust start to Rust end/now.
     */
    elapsed_seconds(): number;
    /**
     * Expected end (JS epoch millis): Rust end once completed,
     * else Rust now + Rust eta while running.
     */
    expected_end_ms(): bigint | undefined;
    /**
     * Evaluates current download state and returns the next scheduling action.
     */
    get_next_action(active_conns: number): any;
    /**
     * Returns a full snapshot of the current download state for the UI.
     */
    get_snapshot(): any;
    /**
     * Marks the whole download completed (wall-clock end time, Rust-owned).
     * Called by the JS coordinator after final assembly, and automatically
     * from mark_segment_completed once every segment is Completed.
     */
    mark_completed(): void;
    /**
     * Marks a segment as completed.
     */
    mark_segment_completed(segment_id: number): void;
    constructor(url_str: string, content_length: bigint | null | undefined, connections: number, scheduler_type_str: string);
    /**
     * Records chunk progress from a worker.
     */
    record_progress(segment_id: number, worker_id: number, chunk_bytes: bigint, duration_ms: number): void;
    /**
     * Remaining seconds from the Rust EWMA eta. None when completed/unknown.
     */
    remaining_seconds(): number | undefined;
    /**
     * Wall-clock start of the engine (JS epoch millis, Rust-owned).
     */
    started_at_ms(): bigint;
}

/**
 * Returns the AURORA engine build version string.
 */
export function aurora_version(): string;

/**
 * Computes a fast BLAKE3 checksum of an in-memory byte buffer.
 */
export function compute_blake3(data: Uint8Array): string;

/**
 * Computes a fast SHA-256 checksum of an in-memory byte buffer.
 */
export function compute_sha256(data: Uint8Array): string;

/**
 * Verifies whether data matches the expected checksum string using SHA256 or BLAKE3.
 */
export function verify_checksum(data: Uint8Array, expected_hex: string, algorithm: string): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_aurorawasmengine_free: (a: number, b: number) => void;
    readonly aurora_version: (a: number) => void;
    readonly aurorawasmengine_completed_at_ms: (a: number, b: number) => void;
    readonly aurorawasmengine_elapsed_seconds: (a: number) => number;
    readonly aurorawasmengine_expected_end_ms: (a: number, b: number) => void;
    readonly aurorawasmengine_get_next_action: (a: number, b: number, c: number) => void;
    readonly aurorawasmengine_get_snapshot: (a: number, b: number) => void;
    readonly aurorawasmengine_mark_completed: (a: number) => void;
    readonly aurorawasmengine_mark_segment_completed: (a: number, b: number) => void;
    readonly aurorawasmengine_new: (a: number, b: number, c: number, d: number, e: bigint, f: number, g: number, h: number) => void;
    readonly aurorawasmengine_record_progress: (a: number, b: number, c: number, d: bigint, e: number) => void;
    readonly aurorawasmengine_remaining_seconds: (a: number, b: number) => void;
    readonly aurorawasmengine_started_at_ms: (a: number) => bigint;
    readonly compute_blake3: (a: number, b: number, c: number) => void;
    readonly compute_sha256: (a: number, b: number, c: number) => void;
    readonly verify_checksum: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
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
