#!/usr/bin/env python3
"""Compare rustdoc's missing-doc diagnostics by source item, not total count."""

import argparse
import collections
import json
from pathlib import Path
import re
import sys

# Strip literals and comments before interpreting braces. Raw strings and
# nested block comments need handling because both may contain Rust syntax.
TOKEN = re.compile(
    r'//[^\n]*|/\*|(?:br|r)(\#*)"|b?"(?:\\.|[^"\\])*"'
    r"|b?'(?:\\.|[^'\\])'|[A-Za-z_][A-Za-z_0-9]*|[^\s]",
    re.DOTALL,
)


def tokens(source):
    offset = 0
    while match := TOKEN.search(source, offset):
        token = match.group()
        offset = match.end()
        if token.startswith('//'):
            continue
        if token == '/*':
            depth = 1
            while depth:
                opening = source.find('/*', offset)
                closing = source.find('*/', offset)
                if closing < 0:
                    raise ValueError('unterminated Rust block comment')
                if 0 <= opening < closing:
                    depth += 1
                    offset = opening + 2
                else:
                    depth -= 1
                    offset = closing + 2
            continue
        if match.group(1) is not None:
            closing = source.find('"' + match.group(1), offset)
            if closing < 0:
                raise ValueError('unterminated Rust raw string')
            offset = closing + 1 + len(match.group(1))
            continue
        if '"' in token or (token.startswith("'") and token.endswith("'")):
            continue
        yield token


def owner_path(source):
    scopes = []
    header = []
    for token in tokens(source):
        if token in ('{', '('):
            owner = None
            for kind in (('struct', 'enum', 'trait', 'mod') if token == '{' else ('struct',)):
                if kind in header:
                    owner = kind + ' ' + header[header.index(kind) + 1]
                    break
            if owner is None and token == '{' and 'impl' in header:
                owner = ' '.join(header[header.index('impl'):])
            if owner is None and scopes and scopes[-1] and scopes[-1].startswith('enum '):
                # Named enum variants with fields own a separate field namespace.
                candidates = [part for part in header if re.fullmatch(r'[A-Z]\w*', part)]
                owner = 'variant ' + candidates[-1] if candidates else None
            scopes.append(owner)
            header = []
        elif token in ('}', ')'):
            if not scopes:
                raise ValueError('unbalanced Rust item scopes')
            scopes.pop()
            header = []
        elif token == ';' or (token == ',' and 'impl' not in header):
            header = []
        else:
            header.append(token)
    return '/'.join(scope for scope in scopes if scope)


def item_key(message):
    span = next(span for span in message['spans'] if span['is_primary'])
    if span.get('expansion'):
        raise ValueError('expanded missing-doc diagnostic needs an explicit item identity')
    source = Path(span['file_name']).read_bytes()
    prefix = source[:span['byte_start']].decode('utf-8')
    declaration = source[span['byte_start']:span['byte_end']].decode('utf-8')
    kind = message['message'].removeprefix('missing documentation for ')
    if kind in ('a function', 'a method', 'an associated function', 'a constant'):
        keyword = 'const' if kind == 'a constant' else 'fn'
        symbol = re.search(r'\b' + keyword + r'\s+(\w+)', declaration).group(1)
    else:
        symbol = re.search(r'(?:pub(?:\([^)]*\))?\s+)?(\w+)', declaration).group(1)
    return f"{span['file_name']}::{owner_path(prefix)}::{kind}::{symbol}"


def missing_items(log):
    items = []
    for line in Path(log).read_text().splitlines():
        record = json.loads(line)
        message = record.get('message')
        if isinstance(message, dict) and (message.get('code') or {}).get('code') == 'missing_docs':
            items.append(item_key(message))
    return collections.Counter(items)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('diagnostics')
    parser.add_argument('baseline')
    parser.add_argument('--write-baseline', action='store_true')
    args = parser.parse_args()
    actual = missing_items(args.diagnostics)
    if args.write_baseline:
        Path(args.baseline).write_text(json.dumps(sorted(actual.elements()), indent=2) + '\n')
        return 0
    expected = collections.Counter(json.loads(Path(args.baseline).read_text()))
    added, resolved = actual - expected, expected - actual
    for key, count in sorted(added.items()):
        print(f'error: newly undocumented item ({count}): {key}', file=sys.stderr)
    for key, count in sorted(resolved.items()):
        print(f'error: remove resolved baseline item ({count}): {key}', file=sys.stderr)
    if added or resolved:
        return 1
    print(f'public documentation debt: {sum(actual.values())} known items (no new undocumented items)')
    return 0


if __name__ == '__main__':
    sys.exit(main())
