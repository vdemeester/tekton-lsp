#!/usr/bin/env python3
import json, subprocess, sys, time, os, re

def parse_lsp_responses(output):
    """Parse LSP protocol responses (Content-Length + JSON)."""
    messages = []
    pattern = r'Content-Length: (\d+)\r\n\r\n'
    
    pos = 0
    while pos < len(output):
        match = re.search(pattern, output[pos:])
        if not match:
            break
        
        length = int(match.group(1))
        json_start = pos + match.end()
        json_end = json_start + length
        
        try:
            msg = json.loads(output[json_start:json_end])
            messages.append(msg)
        except json.JSONDecodeError:
            pass
        
        pos = json_end
    
    return messages

def validate_tekton_file(filepath, lsp_binary="./target/release/tekton-lsp"):
    filepath = os.path.abspath(filepath)
    with open(filepath) as f:
        content = f.read()

    proc = subprocess.Popen(
        [lsp_binary],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )

    def send_msg(msg):
        content_str = json.dumps(msg)
        header = f"Content-Length: {len(content_str)}\r\n\r\n"
        proc.stdin.write((header + content_str).encode())
        proc.stdin.flush()

    # Initialize
    send_msg({"jsonrpc":"2.0","id":1,"method":"initialize",
              "params":{"capabilities":{},"processId":None,
                       "rootUri":f"file://{os.path.dirname(filepath)}"}})
    time.sleep(0.3)

    # Initialized notification
    send_msg({"jsonrpc":"2.0","method":"initialized","params":{}})
    time.sleep(0.3)

    # Send didOpen
    send_msg({"jsonrpc":"2.0","method":"textDocument/didOpen",
              "params":{"textDocument":{"uri":f"file://{filepath}",
                                       "languageId":"yaml","version":1,"text":content}}})
    time.sleep(1)

    proc.stdin.close()
    output = proc.stdout.read().decode()

    # Parse all LSP responses
    messages = parse_lsp_responses(output)
    
    # Find publishDiagnostics message
    for msg in messages:
        if msg.get("method") == "textDocument/publishDiagnostics":
            return msg.get("params", {}).get("diagnostics", [])
    
    return []

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: check-tekton.py <file.yaml>")
        sys.exit(1)

    diagnostics = validate_tekton_file(sys.argv[1])
    if diagnostics:
        print(f"Found {len(diagnostics)} issue(s):")
        for diag in diagnostics:
            line = diag["range"]["start"]["line"] + 1
            severity = {1: "ERROR", 2: "WARNING", 3: "INFO", 4: "HINT"}.get(diag.get("severity", 1), "UNKNOWN")
            print(f"  Line {line} [{severity}]: {diag['message']}")
        sys.exit(1)
    else:
        print("✓ No issues found!")
        sys.exit(0)
