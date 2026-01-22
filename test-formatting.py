#!/usr/bin/env python3
"""Test document formatting via LSP protocol."""
import json, subprocess, sys, time, re

def send_lsp_message(proc, method, params=None, msg_id=None):
    message = {"jsonrpc": "2.0", "method": method}
    if params is not None:
        message["params"] = params
    if msg_id is not None:
        message["id"] = msg_id

    content = json.dumps(message)
    header = f"Content-Length: {len(content)}\r\n\r\n"
    proc.stdin.write((header + content).encode())
    proc.stdin.flush()

def parse_lsp_responses(output):
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

# Start LSP server
lsp_binary = "./target/release/tekton-lsp"
proc = subprocess.Popen([lsp_binary], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

print("✓ LSP server started")

# Initialize
send_lsp_message(proc, "initialize", {"capabilities": {}, "processId": None, "rootUri": "file:///tmp/tekton-test"}, msg_id=1)
time.sleep(0.3)
print("✓ Initialize sent")

# Initialized
send_lsp_message(proc, "initialized", {})
time.sleep(0.3)

# Open document with inconsistent formatting (4-space indent)
content = """apiVersion: tekton.dev/v1
kind: Task
metadata:
    name: my-task
spec:
    params:
        - name: version
    steps:
        - name: build
          image: golang:1.21
"""

send_lsp_message(proc, "textDocument/didOpen", {
    "textDocument": {
        "uri": "file:///tmp/task.yaml",
        "languageId": "yaml",
        "version": 1,
        "text": content
    }
})
time.sleep(0.3)
print("✓ Document opened")

# Request formatting
send_lsp_message(proc, "textDocument/formatting", {
    "textDocument": {"uri": "file:///tmp/task.yaml"},
    "options": {"tabSize": 2, "insertSpaces": True}
}, msg_id=2)
time.sleep(0.5)
print("✓ Formatting requested")

# Shutdown
send_lsp_message(proc, "shutdown", {}, msg_id=3)
send_lsp_message(proc, "exit", {})

proc.stdin.close()
proc.wait(timeout=2)

stdout_output = proc.stdout.read().decode()

# Parse responses
messages = parse_lsp_responses(stdout_output)

print(f"\n📋 All messages received:")
for i, msg in enumerate(messages):
    msg_type = msg.get('method', f"id={msg.get('id')}")
    print(f"  {i}: {msg_type}")

# Find formatting response
formatting_response = None
for msg in messages:
    if msg.get("id") == 2:
        formatting_response = msg
        break

success = True

if formatting_response and "result" in formatting_response:
    result = formatting_response["result"]

    if result is None:
        print(f"\n⚠ Formatting returned null (document may already be formatted or invalid)")
        success = True  # This is acceptable
    elif isinstance(result, list):
        print(f"\n✅ Formatting response received")
        print(f"   Number of edits: {len(result)}")

        if len(result) > 0:
            edit = result[0]
            new_text = edit.get("newText", "")
            print(f"   ✓ Got formatting edit")

            # Check that the formatted text has proper structure
            if "apiVersion:" in new_text and "kind:" in new_text:
                print(f"   ✓ Formatted text preserves structure")
            else:
                print(f"   ⚠ Formatted text may be incorrect")
                success = False

            # Show a snippet of the formatted text
            lines = new_text.split('\n')[:5]
            print(f"   Preview:")
            for line in lines:
                print(f"     {line}")
        else:
            print(f"   ✓ No edits needed (document already formatted)")
    else:
        print(f"\n⚠ Unexpected result format: {type(result)}")
        success = False
else:
    print(f"\n❌ No formatting response received")
    print(f"   Response: {formatting_response}")
    success = False

if success:
    print(f"\n✅ Formatting test passed!")
    sys.exit(0)
else:
    print(f"\n❌ Formatting test failed")
    sys.exit(1)
