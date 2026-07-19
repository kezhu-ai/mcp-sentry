import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    try:
        req = json.loads(line)
    except: continue
    method = req.get("method","")
    req_id = req.get("id")
    if method == "initialize":
        resp = {"jsonrpc":"2.0","id":req_id,"result":{"serverInfo":{"name":"mock","version":"0.0.1"}}}
    elif method == "tools/list":
        resp = {"jsonrpc":"2.0","id":req_id,"result":{"tools":[
            {"name":"read_file","description":"read a file"},
            {"name":"write_file","description":"write a file"},
            {"name":"delete_file","description":"delete a file"},
            {"name":"run_command","description":"run shell"},
        ]}}
    elif method == "tools/call":
        # echo the tool call back as a result (mock server)
        resp = {"jsonrpc":"2.0","id":req_id,"result":{"ok":True,"params":req.get("params")}}
    else:
        resp = {"jsonrpc":"2.0","id":req_id,"error":{"code":-32601,"message":"unknown method"}}
    print(json.dumps(resp))
    sys.stdout.flush()
