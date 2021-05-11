if __name__ == "__main__":
    import json
    from pprint import pprint

    with open("../contracts/HubCulture.abi",'r') as f:
        abi = f.read()
    abis = json.loads(abi)

    function_abis = [f for f in abis if f["type"] == "function"]
    load_functions = {f["name"]:{arg["name"]:"<"+arg["type"]+">" for arg in f["inputs"]} for f in function_abis if f["stateMutability"] == "view"}
    exec_functions = {f["name"]:{arg["name"]:"<"+arg["type"]+">" for arg in f["inputs"]} for f in function_abis if f["stateMutability"] != "view"}

    exec_calls = [{"method":"exec","params":{"name":f,"inputs":exec_functions[f]},"id":"<integer>","jsonrpc":"2.0"} for f in exec_functions]
    load_calls = [{"method":"load","params":{"name":f,"inputs":load_functions[f]},"id":"<integer>","jsonrpc":"2.0"} for f in load_functions]

    print('-------------------------------------')
    print('-------------LOAD CALLS--------------')
    print('-------------------------------------')
    pprint(load_calls)

    print('-------------------------------------')
    print('-------------EXEC CALLS--------------')
    print('-------------------------------------')
    pprint(exec_calls)
