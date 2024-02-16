/* tslint:disable */
/* eslint-disable */
/**
* @param {number} regex
* @returns {number | undefined}
*/
export function regex_to_nfa(regex: number): number | undefined;
/**
* @param {number} dfa
* @returns {boolean}
*/
export function minimize_dfa(dfa: number): boolean;
/**
* @param {number} dfa
* @param {string} canvas_id
* @returns {boolean}
*/
export function draw_dfa(dfa: number, canvas_id: string): boolean;
/**
* @param {number} nfa
* @param {string} canvas_id
* @returns {boolean}
*/
export function draw_nfa(nfa: number, canvas_id: string): boolean;
/**
* @param {number} dfa1
* @param {number} dfa2
* @returns {boolean | undefined}
*/
export function check_dfa_eq(dfa1: number, dfa2: number): boolean | undefined;
/**
* @param {number} nfa1
* @param {number} nfa2
* @returns {boolean | undefined}
*/
export function check_nfa_eq(nfa1: number, nfa2: number): boolean | undefined;
/**
* @param {number} dfa
* @returns {number | undefined}
*/
export function dfa_to_nfa(dfa: number): number | undefined;
/**
* @param {number} nfa
* @returns {number | undefined}
*/
export function nfa_to_dfa(nfa: number): number | undefined;
/**
* @param {number} dfa
* @returns {string | undefined}
*/
export function dfa_to_table(dfa: number): string | undefined;
/**
* @param {number} nfa
* @returns {string | undefined}
*/
export function nfa_to_table(nfa: number): string | undefined;
/**
* @param {number} regex
* @returns {boolean}
*/
export function delete_regex(regex: number): boolean;
/**
* @param {string} input
* @returns {number}
*/
export function load_regex(input: string): number;
/**
* @param {number} dfa
* @returns {boolean}
*/
export function delete_dfa(dfa: number): boolean;
/**
* @param {string} input
* @returns {number}
*/
export function load_dfa(input: string): number;
/**
* @param {number} nfa
* @returns {boolean}
*/
export function delete_nfa(nfa: number): boolean;
/**
* @param {string} input
* @returns {number}
*/
export function load_nfa(input: string): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly regex_to_nfa: (a: number, b: number) => void;
  readonly minimize_dfa: (a: number) => number;
  readonly draw_dfa: (a: number, b: number, c: number) => number;
  readonly draw_nfa: (a: number, b: number, c: number) => number;
  readonly check_dfa_eq: (a: number, b: number) => number;
  readonly check_nfa_eq: (a: number, b: number) => number;
  readonly dfa_to_nfa: (a: number, b: number) => void;
  readonly nfa_to_dfa: (a: number, b: number) => void;
  readonly dfa_to_table: (a: number, b: number) => void;
  readonly nfa_to_table: (a: number, b: number) => void;
  readonly delete_regex: (a: number) => number;
  readonly load_regex: (a: number, b: number, c: number) => void;
  readonly delete_dfa: (a: number) => number;
  readonly load_dfa: (a: number, b: number, c: number) => void;
  readonly delete_nfa: (a: number) => number;
  readonly load_nfa: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
