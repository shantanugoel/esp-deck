// Common keycodes and their human-friendly labels for use with keycode crate
export const keyCodes = [
    // Letters
    ...'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').map(l => ({ code: `Key${l}`, label: l })),
    // Digits
    ...'0123456789'.split('').map(d => ({ code: `Digit${d}`, label: d })),
    // Function keys
    ...Array.from({ length: 12 }, (_, i) => ({ code: `F${i + 1}`, label: `F${i + 1}` })),
    // Arrows
    { code: 'ArrowUp', label: '↑' },
    { code: 'ArrowDown', label: '↓' },
    { code: 'ArrowLeft', label: '←' },
    { code: 'ArrowRight', label: '→' },
    // Modifiers
    { code: 'ControlLeft', label: 'Ctrl (Left)' },
    { code: 'ControlRight', label: 'Ctrl (Right)' },
    { code: 'ShiftLeft', label: 'Shift (Left)' },
    { code: 'ShiftRight', label: 'Shift (Right)' },
    { code: 'AltLeft', label: 'Alt (Left)' },
    { code: 'AltRight', label: 'Alt (Right)' },
    { code: 'MetaLeft', label: 'Meta (Left)' },
    { code: 'MetaRight', label: 'Meta (Right)' },
    // Special
    { code: 'Space', label: 'Space' },
    { code: 'Tab', label: 'Tab' },
    { code: 'Enter', label: 'Enter' },
    { code: 'Escape', label: 'Esc' },
    { code: 'Backspace', label: 'Backspace' },
    { code: 'Delete', label: 'Delete' },
    { code: 'Insert', label: 'Insert' },
    { code: 'Home', label: 'Home' },
    { code: 'End', label: 'End' },
    { code: 'PageUp', label: 'Page Up' },
    { code: 'PageDown', label: 'Page Down' },
    // Symbols
    { code: 'Minus', label: '-' },
    { code: 'Equal', label: '=' },
    { code: 'BracketLeft', label: '[' },
    { code: 'BracketRight', label: ']' },
    { code: 'Backslash', label: '\\' },
    { code: 'Semicolon', label: ';' },
    { code: 'Quote', label: "'" },
    { code: 'Comma', label: ',' },
    { code: 'Period', label: '.' },
    { code: 'Slash', label: '/' },
    { code: 'Backquote', label: '`' },
]

// Helper: Map from character to keycode and modifier
const charToKey = (char: string): { key: string, modifier: string } => {
    // Letters
    if (/^[a-zA-Z]$/.test(char)) {
        const upper = char.toUpperCase()
        return {
            key: `Key${upper}`,
            modifier: char === upper ? 'ShiftLeft' : ''
        }
    }
    // Digits
    if (/^[0-9]$/.test(char)) {
        return { key: `Digit${char}`, modifier: '' }
    }
    // Space
    if (char === ' ') return { key: 'Space', modifier: '' }
    // Symbols (US QWERTY)
    const symbolMap: Record<string, { key: string, modifier?: string }> = {
        '!': { key: 'Digit1', modifier: 'ShiftLeft' },
        '@': { key: 'Digit2', modifier: 'ShiftLeft' },
        '#': { key: 'Digit3', modifier: 'ShiftLeft' },
        '$': { key: 'Digit4', modifier: 'ShiftLeft' },
        '%': { key: 'Digit5', modifier: 'ShiftLeft' },
        '^': { key: 'Digit6', modifier: 'ShiftLeft' },
        '&': { key: 'Digit7', modifier: 'ShiftLeft' },
        '*': { key: 'Digit8', modifier: 'ShiftLeft' },
        '(': { key: 'Digit9', modifier: 'ShiftLeft' },
        ')': { key: 'Digit0', modifier: 'ShiftLeft' },
        '-': { key: 'Minus' },
        '_': { key: 'Minus', modifier: 'ShiftLeft' },
        '=': { key: 'Equal' },
        '+': { key: 'Equal', modifier: 'ShiftLeft' },
        '[': { key: 'BracketLeft' },
        '{': { key: 'BracketLeft', modifier: 'ShiftLeft' },
        ']': { key: 'BracketRight' },
        '}': { key: 'BracketRight', modifier: 'ShiftLeft' },
        '\\': { key: 'Backslash' },
        '|': { key: 'Backslash', modifier: 'ShiftLeft' },
        ';': { key: 'Semicolon' },
        ':': { key: 'Semicolon', modifier: 'ShiftLeft' },
        "'": { key: 'Quote' },
        '"': { key: 'Quote', modifier: 'ShiftLeft' },
        ',': { key: 'Comma' },
        '<': { key: 'Comma', modifier: 'ShiftLeft' },
        '.': { key: 'Period' },
        '>': { key: 'Period', modifier: 'ShiftLeft' },
        '/': { key: 'Slash' },
        '?': { key: 'Slash', modifier: 'ShiftLeft' },
        '`': { key: 'Backquote' },
        '~': { key: 'Backquote', modifier: 'ShiftLeft' },
    }
    if (char in symbolMap) {
        const { key, modifier } = symbolMap[char]
        return { key, modifier: modifier || '' }
    }
    // Fallback: try to find by label
    const found = keyCodes.find(kc => kc.label === char)
    if (found) return { key: found.code, modifier: '' }
    // Unknown
    return { key: '', modifier: '' }
}

export function stringToKeyCodes(text: string): { keys: string[], modifiers: string[] } {
    const keys: string[] = []
    const modifiers: string[] = []
    for (const char of text) {
        const { key, modifier } = charToKey(char)
        keys.push(key)
        modifiers.push(modifier)
    }
    return { keys, modifiers }
}

export function keyCodesToString(keys: string[], modifiers: string[]): string {
    // Only supports basic US QWERTY for now
    const reverseMap: Record<string, string> = {
        KeyA: 'a', KeyB: 'b', KeyC: 'c', KeyD: 'd', KeyE: 'e', KeyF: 'f', KeyG: 'g', KeyH: 'h', KeyI: 'i', KeyJ: 'j', KeyK: 'k', KeyL: 'l', KeyM: 'm', KeyN: 'n', KeyO: 'o', KeyP: 'p', KeyQ: 'q', KeyR: 'r', KeyS: 's', KeyT: 't', KeyU: 'u', KeyV: 'v', KeyW: 'w', KeyX: 'x', KeyY: 'y', KeyZ: 'z',
        Digit0: '0', Digit1: '1', Digit2: '2', Digit3: '3', Digit4: '4', Digit5: '5', Digit6: '6', Digit7: '7', Digit8: '8', Digit9: '9',
        Space: ' ', Minus: '-', Equal: '=', BracketLeft: '[', BracketRight: ']', Backslash: '\\', Semicolon: ';', Quote: "'", Comma: ',', Period: '.', Slash: '/', Backquote: '`',
    }
    const shiftMap: Record<string, string> = {
        KeyA: 'A', KeyB: 'B', KeyC: 'C', KeyD: 'D', KeyE: 'E', KeyF: 'F', KeyG: 'G', KeyH: 'H', KeyI: 'I', KeyJ: 'J', KeyK: 'K', KeyL: 'L', KeyM: 'M', KeyN: 'N', KeyO: 'O', KeyP: 'P', KeyQ: 'Q', KeyR: 'R', KeyS: 'S', KeyT: 'T', KeyU: 'U', KeyV: 'V', KeyW: 'W', KeyX: 'X', KeyY: 'Y', KeyZ: 'Z',
        Digit1: '!', Digit2: '@', Digit3: '#', Digit4: '$', Digit5: '%', Digit6: '^', Digit7: '&', Digit8: '*', Digit9: '(', Digit0: ')',
        Minus: '_', Equal: '+', BracketLeft: '{', BracketRight: '}', Backslash: '|', Semicolon: ':', Quote: '"', Comma: '<', Period: '>', Slash: '?', Backquote: '~',
    }
    let result = ''
    for (let i = 0; i < keys.length; i++) {
        const key = keys[i]
        const mod = modifiers[i]
        if (mod && mod.includes('ShiftLeft') && key in shiftMap) {
            result += shiftMap[key]
        } else if (key in reverseMap) {
            result += reverseMap[key]
        } else {
            result += '?'
        }
    }
    return result
} 