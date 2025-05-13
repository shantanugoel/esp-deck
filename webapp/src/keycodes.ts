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