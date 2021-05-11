"""Template Generator.
Usage:  template_generator.py <abi-file> [ --origin O ]  [ --transport T ] [ --host HOST ] [ --port PORT ] [ --route R ]
        template_generator.py --help
        template_generator.py --version


Flags:
--help                   Show this screen.
--version                Print the program version.
--transport T            HTTP or HTTPS[default: http]
--host HOST              IP address OR dns name of blockchain node [default: 127.0.0.1]
--port PORT              Port of webhook server [default: 8080]
--route R                Route of the callback.[default: /]
--origin O               Origin address of contract.[default: []]
"""

if __name__ == "__main__":
    import json
    import docopt

    kwargs = docopt.docopt(__doc__,version='0.0.1')

    transport = kwargs['--transport']
    host = kwargs['--host']
    port = kwargs['--port']
    route = kwargs['--route']
    abi = kwargs['<abi-file>']
    origin = kwargs['--origin']

    with open(abi,'r') as f:
        abi = f.read()
    abis = json.loads(abi)

    event_abis = [f for f in abis if f["type"] == "event"]

    event_configs = [ {
                            "name":event["name"],
                            "template":"{}.tera".format(event["name"]),
                            "inputs":event["inputs"],
                            "type":event["type"],
                        }
                        for event in event_abis ]

    callback_config = {
        "callback" : "{}://{}:{}{}".format(transport,host,port,route),
        "origin" : '{}'.format(json.loads(origin)),
        "events" : [ event["name"] for event in event_configs]
        }

    #TODO add event hosting config

    with open('config.toml','w') as f:
        f.write('[[callback-config]]')
        f.write('\n')
        f.write('callback = "{}"'.format(callback_config["callback"]))
        f.write('\n')
        f.write('origin = {}'.format(callback_config["origin"]))
        f.write('\n')
        f.write('events = [')
        f.write('\n')
        for event in callback_config["events"]:
            f.write('    "{}",'.format(event))
            f.write('\n')
        f.write(']')
        f.write('\n')
        for event in event_configs:
            f.write('\n')
            f.write('[[event-config]]')
            f.write('\n')
            f.write('template = "{}"'.format(event["template"]))
            f.write('\n')
            f.write('name = "{}"'.format(event["name"]))
            f.write('\n')
            f.write('inputs = [')
            f.write('\n')
            for inpt in event["inputs"]:
                f.write('    { ')
                f.write('name = "{}", type = "{}", indexed = {}'.format(inpt["name"],inpt["type"],"true" if inpt["indexed"] else "false"))
                f.write(' },')
                f.write('\n')
            f.write(']')
            f.write('\n')

    for event in event_configs:
        with open('./templates/{}.tera'.format(event["name"]),'w') as f:
            f.write('{')
            f.write('\n')
            f.write('    ')
            f.write('"event": ')
            f.write('{{ ')
            f.write('meta.event_name')
            f.write(' | json_encode() ')
            f.write('}},')
            f.write('\n')
            f.write('    ')
            f.write('"origin": ')
            f.write('{{ ')
            f.write('log.address')
            f.write(' | json_encode() ')
            f.write('}},')
            f.write('\n')
            f.write('    ')
            f.write('"block": ')
            f.write('{{ ')
            f.write('log.blockNumber')
            f.write(' | json_encode() ')
            f.write('}},')
            length = len(event["inputs"])
            for i,inpt in enumerate(event["inputs"]):
                f.write('\n')
                f.write('    ')
                f.write('"{}": '.format(inpt["name"]))
                f.write('{{ ')
                f.write('event.{}'.format(inpt["name"]))
                f.write(' | json_encode() ')
                if i == length-1:
                    f.write('}}')
                else:
                    f.write('}},')
            f.write('\n')
            f.write('}')
