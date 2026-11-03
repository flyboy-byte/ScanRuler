"use strict";
// SPDX-License-Identifier: AGPL-3.0-only
// What a command is: one verb of the session, typed for something other than
// a mouse to call — an agent through the bridge, a script, a test. The verbs
// are the panels' own; a command only names one, checks its input, runs it
// as one step of the undo history and reads back what the stores hold
// afterwards. See registry.ts for how they are found and run.
Object.defineProperty(exports, "__esModule", { value: true });
exports.errorMessage = exports.CommandError = void 0;
class CommandError extends Error {
    code;
    constructor(code, message) {
        super(message);
        this.code = code;
        this.name = 'CommandError';
    }
    toJSON() {
        return { code: this.code, message: this.message };
    }
}
exports.CommandError = CommandError;
/** What a caught error says, whatever was thrown. */
const errorMessage = (e) => (e instanceof Error ? e.message : String(e));
exports.errorMessage = errorMessage;
//# sourceMappingURL=types.js.map