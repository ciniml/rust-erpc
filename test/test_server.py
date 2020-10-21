import sys
import erpc
from erpc_shim import rpc_ble_api

class HostHandler(object):
    def __init__(self):
        pass

    def rpc_ble_init(self):
        print("rpc_ble_init")
    
    def rpc_ble_start(self):
        print("rpc_ble_start")
    
    def rpc_ble_deinit(self):
        print("rpc_ble_deinit")


def main():
    handler = HostHandler()

    transport = erpc.transport.TCPTransport("localhost", 5555, True)
    service = rpc_ble_api.server.rpc_ble_hostService(handler)
    server = erpc.simple_server.SimpleServer(transport, erpc.basic_codec.BasicCodec)
    server.add_service(service)
    print("Starting BLE Host Service.")
    sys.stdout.flush()

    server.run()

main()
